//! Access the page tables through a recursively mapped four-or-five-level root table.

use core::fmt;

use super::*;
use crate::addr::{
    FixedValidity, RuntimeValidity, VirtAddr48, VirtAddrGeneric, VirtAddrRT, VirtAddrValidity,
    canonicalize_with_bits,
};
use crate::registers::control::Cr3;
use crate::structures::paging::{
    PageTableIndex, PageTableRootLevel,
    page::AddressNotAligned,
    page_table::{FrameError, PageTable, PageTableEntry, PageTableIndices},
};

/// A currently-active recursive page table.
///
/// A recursive page table is a root level page table with an entry mapped to the table itself.
/// Such a recursive entry allows accessing the page table and all its lower level page tables by
/// constructing special virtual addresses that the page table itself translates. For
/// example, for a four-level page table:
///
/// - To access the level 4 page table, we “loop” (i.e. follow the recursively mapped entry) four
///   times.
/// - To access a level 3 page table, we “loop” three times and then use the level 4 index.
/// - To access a level 2 page table, we “loop” two times, then use the level 4 index, then the
///   level 3 index.
/// - To access a level 1 page table, we “loop” once, then use the level 4 index, then the
///   level 3 index, then the level 2 index.
///
/// It's also required that:
///
/// - The page table must be active, i.e. the CR3 register must contain its physical address.
/// - The level of the page table must match the current active root level.
/// - The page table must have one recursive entry, i.e. an entry that points to the table
///   itself.
///   - The reference must use that “loop”, i.e. be of the form `0o_xxx_xxx_xxx_xxx_0000` where
///     `xxx` is the recursive entry in four-level page tables and `0o_xxx_xxx_xxx_xxx_xxx_0000`
///     in five-level page tables.
/// - The recursive index must not be 511, i.e. the last entry in the page table.
///   - It's unsound because allocating the last byte of the address space can lead to pointer
///     overflows and undefined behavior. For more details, see the discussions
///     [on Zulip](https://rust-lang.zulipchat.com/#narrow/stream/136281-t-opsem/topic/end-of-address-space)
///     and [in the `unsafe-code-guidelines` repo](https://github.com/rust-lang/unsafe-code-guidelines/issues/420).
///
/// This struct implements the `Mapper` trait. Newly allocated parent entries always receive
/// `PRESENT` and `WRITABLE`, as required by recursive paging. Existing parent entries have the
/// requested parent flags merged into their current flags.
///
/// This struct relies on the `CR4.LA57` cache maintained by [`VirtAddrRT`]. [`Self::new`],
/// [`Self::new_with_level`] and [`Self::new_unchecked`] refresh it with
/// [`VirtAddrRT::refetch_virtual_address_bits`]. The explicit-level unchecked constructors
/// and other methods do not refresh it. After changing `CR4.LA57`,
/// the caller must refresh the cache and reestablish the active-root and recursive-mapping
/// invariants before using the mapper. Refreshing the cache does not change its saved root level.
///
/// ```no_run
/// use x86_64::structures::paging::{PageTable, PageTableRootLevel, RecursivePageTable};
///
/// # fn example(root: &mut PageTable) {
/// let mapper = RecursivePageTable::new(root).unwrap();
/// let _: PageTableRootLevel = mapper.root_level();
/// let root: &PageTable = mapper.root_table();
/// # let _ = root;
/// # }
/// ```
#[derive(Debug)]
pub struct RecursivePageTable<'a> {
    /// The active root, borrowed through its recursive mapping.
    root_table: &'a mut PageTable,
    /// The root entry that points back to the root's own frame.
    recursive_index: PageTableIndex,
    /// The root level selected at construction, not a traversal cursor.
    root_level: PageTableRootLevel,
}

impl<'a> RecursivePageTable<'a> {
    /// Creates a new [`RecursivePageTable`] from the reference to the current active root table.
    ///
    /// For required invariants, see the documentation of the [`RecursivePageTable`] type.
    ///
    /// This method refreshes the `CR4.LA57` cache, then checks that every address index from P1
    /// through the active root level equals the recursive index and that the recursive entry
    /// points to the current CR3 frame. Index 511 is rejected with
    /// [`InvalidPageTable::RecursiveIndexUnavailable`]. Use [`Self::new_with_level`] to also
    /// check an explicitly requested root level against the active mode.
    ///
    /// This method reads privileged registers and must execute in Ring 0. The recursive
    /// mapping and active mode must remain valid while the mapper is used. The pointer boundary
    /// restriction for recursive index 511 described on this type still applies: rejecting
    /// that index does not make creating an invalid input reference safe.
    #[inline]
    pub fn new(root_table: &'a mut PageTable) -> Result<Self, InvalidPageTable> {
        Self::new_checked(root_table, None)
    }

    /// Creates an active recursive mapper, checking the requested root level against CR4.LA57.
    ///
    /// Refreshes the runtime address-width cache and returns [`InvalidPageTable::ModeMismatch`]
    /// if `root_level` differs from the active mode, before inspecting the recursive entry.
    /// Otherwise performs the same address, index and CR3 checks as [`Self::new`].
    /// The requirements of [`Self::new`] apply, including Ring 0 execution and a valid input
    /// reference. The level of a page table cannot be inferred from its contents or address.
    #[inline]
    pub fn new_with_level(
        root_table: &'a mut PageTable,
        root_level: PageTableRootLevel,
    ) -> Result<Self, InvalidPageTable> {
        Self::new_checked(root_table, Some(root_level))
    }

    /// Shared checked construction; an absent requested level selects the active root.
    fn new_checked(
        root_table: &'a mut PageTable,
        requested_level: Option<PageTableRootLevel>,
    ) -> Result<Self, InvalidPageTable> {
        VirtAddrRT::refetch_virtual_address_bits();
        let root_level = requested_level.unwrap_or_else(active_root_level);

        let address = root_table as *const PageTable as u64;
        let recursive_index =
            classify_recursive_address(address, root_level, RuntimeValidity::bits())?;

        // The address and index were checked before accessing the recursive entry.
        if root_table[recursive_index].frame(false) != Ok(Cr3::read().0) {
            return Err(InvalidPageTable::NotActive);
        }

        Ok(Self {
            root_table,
            recursive_index,
            root_level,
        })
    }

    /// Creates a new [`RecursivePageTable`] without performing any checks.
    ///
    /// This method still refreshes and uses the `CR4.LA57` cache. Use [`Self::new_l4_unchecked`]
    /// or [`Self::new_l5_unchecked`] if that's not desired.
    ///
    /// # Safety
    ///
    /// The table must be the active CR3 root with a level matching CR4.LA57, and the reference
    /// must use its recursive mapping. The `recursive_index` parameter must identify its
    /// self-referential entry and must not be 511, as required by this type's pointer-boundary
    /// restrictions. These requirements are not checked.
    #[inline]
    pub unsafe fn new_unchecked(
        root_table: &'a mut PageTable,
        recursive_index: PageTableIndex,
    ) -> Self {
        VirtAddrRT::refetch_virtual_address_bits();
        Self {
            root_table,
            recursive_index,
            root_level: active_root_level(),
        }
    }

    /// Creates a new four-level [`RecursivePageTable`] without performing any checks.
    ///
    /// This method does not refresh the `CR4.LA57` cache.
    ///
    /// # Safety
    ///
    /// `CR4.LA57` must be `0`, and all requirements of [`Self::new_unchecked`] apply.
    #[inline]
    pub unsafe fn new_l4_unchecked(
        root_table: &'a mut PageTable,
        recursive_index: PageTableIndex,
    ) -> Self {
        Self {
            root_table,
            recursive_index,
            root_level: PageTableRootLevel::Four,
        }
    }

    /// Creates a new five-level [`RecursivePageTable`] without performing any checks.
    ///
    /// This method does not refresh the `CR4.LA57` cache.
    ///
    /// # Safety
    ///
    /// `CR4.LA57` must be `1`, and all requirements of [`Self::new_unchecked`] apply.
    #[inline]
    pub unsafe fn new_l5_unchecked(
        root_table: &'a mut PageTable,
        recursive_index: PageTableIndex,
    ) -> Self {
        Self {
            root_table,
            recursive_index,
            root_level: PageTableRootLevel::Five,
        }
    }

    /// Returns the wrapped root table.
    pub fn root_table(&self) -> &PageTable {
        self.root_table
    }

    /// Returns the wrapped root table mutably.
    ///
    /// The recursive entry and the active hierarchy must continue to satisfy the construction
    /// invariants before any other mapper method is used.
    pub fn root_table_mut(&mut self) -> &mut PageTable {
        self.root_table
    }

    /// Returns the root level selected at construction.
    pub const fn root_level(&self) -> PageTableRootLevel {
        self.root_level
    }

    /// Returns target indices after checking both the address policy and root representability.
    ///
    /// L4 roots require VA48 addresses; L5 roots use the cached runtime width. Either failed
    /// check returns [`WalkError::AddressNotValid`]. This does not access any page table or
    /// explicitly refresh the runtime cache, and the returned indices carry no root or level.
    fn validated_indices<V: VirtAddrValidity>(
        &self,
        address: VirtAddrGeneric<V>,
    ) -> Result<PageTableIndices, WalkError> {
        VirtAddrGeneric::<V>::try_new_with_validity(address.as_u64())
            .map_err(|_| WalkError::AddressNotValid)?;
        let representable = match self.root_level {
            PageTableRootLevel::Four => VirtAddr48::try_new(address.as_u64()).is_ok(),
            PageTableRootLevel::Five => VirtAddrRT::try_new(address.as_u64()).is_ok(),
        };
        if !representable {
            return Err(WalkError::AddressNotValid);
        }
        Ok(PageTableIndices::from_address(address.as_u64()))
    }

    /// Builds the uncanonicalized, page-aligned address of a child table's recursive window.
    ///
    /// The window begins with `table_level` repeated recursive indices, followed by the target's
    /// ancestor indices from the root down. For example, a P2 window is `[r,r,a4,a3]` under L4
    /// and `[r,r,a5,a4,a3]` under L5. No target validity or runtime cache is consulted.
    ///
    /// Returns `None` for a table at or above the root, or if checked address construction fails.
    /// The root itself is accessed directly; levels above it do not exist in this hierarchy.
    /// The caller canonicalizes the result for the active mode; pure tests can instead use an
    /// explicit bit width without reading CR4.
    fn recursive_window_offset(
        root_level: PageTableRootLevel,
        recursive_index: PageTableIndex,
        indices: &PageTableIndices,
        table_level: PageTableLevel,
    ) -> Option<u64> {
        let root = root_level.as_page_table_level();
        if table_level >= root {
            return None; // The root is accessed directly through root_table.
        }
        let loops = table_level as usize;
        let mut window_level = root;
        let mut ancestor = root;
        let mut offset = 0u64;
        for position in 0..root as usize {
            let index = if position < loops {
                recursive_index
            } else {
                let index = indices.index(ancestor);
                ancestor = ancestor.next_lower_level()?;
                index
            };
            offset = offset.checked_add(
                u64::from(index).checked_mul(window_level.entry_address_space_alignment())?,
            )?;
            if let Some(next) = window_level.next_lower_level() {
                window_level = next;
            }
        }
        Some(offset)
    }

    /// Identifies the root's recursive entry, never a same-numbered entry in a lower table.
    fn is_root_recursive_entry(
        root_level: PageTableRootLevel,
        recursive_index: PageTableIndex,
        level: PageTableLevel,
        index: usize,
    ) -> bool {
        level == root_level.as_page_table_level() && index == usize::from(recursive_index)
    }

    /// Returns the runtime-canonical window address for the target's table at `table_level`.
    ///
    /// The window is always canonicalized with the cached active [`RuntimeValidity`]. Its mode
    /// must agree with this root and CR4.LA57. A mode switch requires an explicit refetch
    /// by the caller and reestablishment of the mapper's construction safety invariants.
    /// No target validity is substituted for the runtime policy, and this method does not
    /// explicitly refresh the cache. It panics if the requested table is at or above the root
    /// or if window-address construction fails. Constructing the address does not dereference it.
    fn recursive_window_address(
        root_level: PageTableRootLevel,
        recursive_index: PageTableIndex,
        indices: &PageTableIndices,
        table_level: PageTableLevel,
    ) -> VirtAddrRT {
        let offset =
            Self::recursive_window_offset(root_level, recursive_index, indices, table_level)
                .expect("a child table has a representable recursive window");
        VirtAddrRT::new_truncate(offset)
    }

    /// Checks a parent entry and borrows its child through the corresponding recursive window.
    ///
    /// The caller must pair `entry` with the target `indices` and the entry's immediate
    /// `child_level` in the active hierarchy selected by `root_level` and `recursive_index`.
    /// `access` checks the entry but cannot establish that correspondence or the mapping
    /// invariants. The returned borrow is bounded by `entry`, not by `indices`; a failed parent
    /// check is returned before building or accessing a window.
    fn next_table<'b>(
        root_level: PageTableRootLevel,
        recursive_index: PageTableIndex,
        entry: &'b PageTableEntry,
        indices: &PageTableIndices,
        child_level: PageTableLevel,
        access: ParentAccess,
    ) -> Result<&'b PageTable, WalkError> {
        access.check(entry)?;
        // SAFETY: the parent entry was checked; the construction invariants guarantee that
        // these indices and child level identify a mapped child table and the active mode/cache agree.
        Ok(unsafe {
            table_from_window(
                entry,
                Self::recursive_window_address(root_level, recursive_index, indices, child_level),
            )
        })
    }

    /// Mutably borrows a child under the same pairing requirements as [`Self::next_table`].
    ///
    /// The window must be writable and the exclusive parent borrow must permit an exclusive
    /// child borrow. The result cannot outlive `entry`; parent-check errors precede window access.
    fn next_table_mut<'b>(
        root_level: PageTableRootLevel,
        recursive_index: PageTableIndex,
        entry: &'b mut PageTableEntry,
        indices: &PageTableIndices,
        child_level: PageTableLevel,
        access: ParentAccess,
    ) -> Result<&'b mut PageTable, WalkError> {
        access.check(entry)?;
        // SAFETY: as above; the exclusive parent borrow bounds the child borrow.
        Ok(unsafe {
            table_from_window_mut(
                entry,
                Self::recursive_window_address(root_level, recursive_index, indices, child_level),
            )
        })
    }

    /// Returns a child table, allocating and clearing a frame if the parent entry is unused.
    ///
    /// A new entry receives `PRESENT | WRITABLE | insert_flags`; an existing entry has only
    /// `insert_flags` merged into its flags. `FrameAllocationFailed` reports allocator exhaustion,
    /// and `ParentEntryHugePage` reports a huge entry after the flag update. Existing tables
    /// are not cleared. The returned borrow is bounded by the parent entry, not by `indices`.
    ///
    /// # Safety
    ///
    /// `indices` and `child_level` must identify this entry's immediate child in the active
    /// recursive hierarchy. Its window must be accessible, writable, aligned and free of
    /// conflicting aliases. Allocated frames and parent permission changes must be safe for
    /// this hierarchy, and the root level, runtime cache and active mode must agree.
    unsafe fn create_next_table<'b, S: PageSize, A: FrameAllocator<Size4KiB> + ?Sized>(
        root_level: PageTableRootLevel,
        recursive_index: PageTableIndex,
        entry: &'b mut PageTableEntry,
        indices: &PageTableIndices,
        child_level: PageTableLevel,
        insert_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<&'b mut PageTable, MapToError<S>> {
        let created = if entry.is_unused() {
            let frame = allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            entry.set_frame(
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | insert_flags,
            );
            true
        } else {
            if !insert_flags.is_empty() && !entry.flags().contains(insert_flags) {
                entry.set_flags(entry.flags() | insert_flags);
            }
            false
        };
        if entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            return Err(MapToError::ParentEntryHugePage);
        }
        // SAFETY: inherited from this method's child indices/level/window preconditions.
        let table = unsafe {
            table_from_window_mut(
                entry,
                Self::recursive_window_address(root_level, recursive_index, indices, child_level),
            )
        };
        if created {
            table.zero();
        }
        Ok(table)
    }

    /// Borrows the table at `target_level` along the target address's path from this root.
    ///
    /// `indices` must come from an address validated for this mapper, and `target_level` must
    /// not exceed the root level. Each parent is checked according to `access`; a failed
    /// check returns a [`WalkError::Frame`]. The root itself is returned without a parent check
    /// or window access. The result is tied to the mapper borrow, not the indices borrow.
    fn table(
        &self,
        indices: &PageTableIndices,
        target_level: PageTableLevel,
        access: ParentAccess,
    ) -> Result<&PageTable, WalkError> {
        let mut table = &*self.root_table;
        let mut current_level = self.root_level.as_page_table_level();
        while current_level != target_level {
            let entry = &table[indices.index(current_level)];
            let child_level = current_level
                .next_lower_level()
                .expect("target table must be below the current table");
            table = Self::next_table(
                self.root_level,
                self.recursive_index,
                entry,
                indices,
                child_level,
                access,
            )?;
            current_level = child_level;
        }
        Ok(table)
    }

    /// Mutably borrows a table using the same path and parent checks as [`Self::table`].
    ///
    /// The same validated-indices and target-level requirements apply. The returned table is
    /// exclusively borrowed for at most the lifetime of the mutable mapper borrow.
    fn table_mut(
        &mut self,
        indices: &PageTableIndices,
        target_level: PageTableLevel,
        access: ParentAccess,
    ) -> Result<&mut PageTable, WalkError> {
        let mut table = &mut *self.root_table;
        let mut current_level = self.root_level.as_page_table_level();
        while current_level != target_level {
            let entry = &mut table[indices.index(current_level)];
            let child_level = current_level
                .next_lower_level()
                .expect("target table must be below the current table");
            table = Self::next_table_mut(
                self.root_level,
                self.recursive_index,
                entry,
                indices,
                child_level,
                access,
            )?;
            current_level = child_level;
        }
        Ok(table)
    }

    /// Returns the target table, allocating missing tables along the path from the root.
    ///
    /// Each descent uses [`Self::create_next_table`] to merge parent flags and
    /// initialize newly allocated tables. Allocation failure or a huge parent entry is returned
    /// as a mapping error; earlier allocations and flag changes are not rolled back.
    ///
    /// # Safety
    ///
    /// `indices` must describe an address validated for this root, and `target_level` must not
    /// exceed the root level. The active recursive mapping must make every child window
    /// accessible and writable when needed. Allocated frames must be safe to use as page tables,
    /// and changing parent permissions or borrowing child tables must not violate alias safety.
    unsafe fn create_table<S: PageSize, A: FrameAllocator<Size4KiB> + ?Sized>(
        &mut self,
        indices: &PageTableIndices,
        target_level: PageTableLevel,
        parent_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<&mut PageTable, MapToError<S>> {
        let mut table = &mut *self.root_table;
        let mut current_level = self.root_level.as_page_table_level();
        while current_level != target_level {
            let entry = &mut table[indices.index(current_level)];
            let child_level = current_level
                .next_lower_level()
                .expect("target table must be below the current table");
            // SAFETY: the indices and child level identify this parent entry's child; the mapper's
            // active recursive mapping and map_to safety contract make its window accessible.
            table = unsafe {
                Self::create_next_table(
                    self.root_level,
                    self.recursive_index,
                    entry,
                    indices,
                    child_level,
                    parent_flags,
                    allocator,
                )?
            };
            current_level = child_level;
        }
        Ok(table)
    }

    /// Replaces the flags of the selected ancestor entry and returns an all-pages flush token.
    ///
    /// Validates the target address first. Levels above the root return `PageTableLevelNotPresent`;
    /// levels at or below the leaf for `S` return `ParentEntryHugePage`. Traversal uses
    /// [`ParentAccess::NonUnused`], and an unused selected entry returns `PageNotMapped`.
    /// This internal helper relies on the public unsafe caller to ensure permission changes
    /// cannot violate memory safety. It does not flush the TLB itself.
    fn set_parent_flags<S: PageSize, V: VirtAddrValidity>(
        &mut self,
        page: Page<S, V>,
        level: PageTableLevel,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let indices = self
            .validated_indices(page.start_address())
            .map_err(WalkError::flags)?;
        if level > self.root_level.as_page_table_level() {
            return Err(FlagUpdateError::PageTableLevelNotPresent);
        }
        if level <= leaf_level::<S>() {
            return Err(FlagUpdateError::ParentEntryHugePage);
        }
        let table = self
            .table_mut(&indices, level, ParentAccess::NonUnused)
            .map_err(WalkError::flags)?;
        let entry = &mut table[indices.index(level)];
        if entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        entry.set_flags(flags);
        Ok(MapperFlushAll::new())
    }

    /// Cleans the inclusive page range containing `start` and `end`, starting at the root.
    ///
    /// `R` supplies the root's traversal width, independently of the public range's policy:
    /// VA48 for L4, runtime validity for L5. The logical root base is zero, so canonical-aware
    /// forwarding can cover both halves in one range. Invalid endpoints or failed address
    /// arithmetic panic as invariant violations rather than being treated as nonempty tables.
    ///
    /// # Safety
    ///
    /// The endpoints must form a nonempty range valid for `R` and this root. The active root,
    /// runtime cache and recursive windows must agree. As required by [`CleanUp`], any freed
    /// table frame must have no other aliases or uses, including in other address spaces.
    unsafe fn clean_up_range<R: VirtAddrValidity, D: FrameDeallocator<Size4KiB>>(
        &mut self,
        start: u64,
        end: u64,
        frame_deallocator: &mut D,
    ) {
        let start = VirtAddrGeneric::<R>::try_new_with_validity(start)
            .expect("cleanup start must be representable by the root");
        let end = VirtAddrGeneric::<R>::try_new_with_validity(end)
            .expect("cleanup end must be representable by the root");
        self.validated_indices(start)
            .expect("cleanup start must be valid for this root");
        self.validated_indices(end)
            .expect("cleanup end must be valid for this root");
        let range = Page::<Size4KiB, R>::range_inclusive(
            Page::containing_address(start),
            Page::containing_address(end),
        );
        // SAFETY: inherited alias safety and deallocation requirements from CleanUp.
        unsafe {
            clean_up_table(
                self.root_table,
                self.root_level,
                self.recursive_index,
                self.root_level.as_page_table_level(),
                VirtAddrGeneric::<R>::zero(),
                range,
                frame_deallocator,
            )
        }
        .expect("cleanup address arithmetic must remain inside the root's canonical space");
    }
}

/// Classifies the address shape required for an active recursive root.
///
/// Checks the requested root against the active width before inspecting the raw address.
/// Address shape alone cannot establish the root level. The address must be canonical under
/// the active width and repeat one index through the selected root. Index 511 is rejected
/// because a `PageTable` at the last page violates the pointer-boundary restriction.
/// This helper never creates or dereferences a pointer; `active_bits` is 48 or 57.
fn classify_recursive_address(
    address: u64,
    root_level: PageTableRootLevel,
    active_bits: usize,
) -> Result<PageTableIndex, InvalidPageTable> {
    let requested_bits = match root_level {
        PageTableRootLevel::Four => 48,
        PageTableRootLevel::Five => 57,
    };
    if requested_bits != active_bits {
        return Err(InvalidPageTable::ModeMismatch);
    }
    if canonicalize_with_bits(address, active_bits) != address {
        return Err(InvalidPageTable::NotActive);
    }
    let recursive_index = PageTableIndices::from_address(address)
        .recursive_index(root_level)
        .ok_or(InvalidPageTable::NotRecursive)?;
    if recursive_index == PageTableIndex::new(511) {
        return Err(InvalidPageTable::RecursiveIndexUnavailable);
    }
    Ok(recursive_index)
}

/// Returns the root level from runtime validity after the automatic constructors refresh it.
///
/// This helper does not explicitly refresh the cache. Runtime validity can initialize an empty
/// cache by reading CR4, so callers must not use it as a register-free mode query in host tests.
fn active_root_level() -> PageTableRootLevel {
    PageTableRootLevel::current_active_root_level()
}

impl PageTableIndices {
    /// Returns the common index if P1 through the specified root all contain the same index.
    ///
    /// Indices above the root are ignored. This checks only the recursive address shape, not
    /// canonicality, alignment, CR3 or the contents of the recursive entry. Index 511 is not
    /// rejected here; the mapper's pointer-boundary restrictions still apply.
    fn recursive_index(&self, root_level: PageTableRootLevel) -> Option<PageTableIndex> {
        let root = root_level.as_page_table_level();
        let recursive_index = self.index(root);
        let mut level = root;
        while let Some(lower) = level.next_lower_level() {
            if self.index(lower) != recursive_index {
                return None;
            }
            level = lower;
        }
        Some(recursive_index)
    }
}

/// Converts a recursive-window address into a shared child-table borrow bounded by `_parent`.
///
/// # Safety
///
/// `address` must point to `_parent`'s actual child in the active hierarchy, with the runtime
/// cache and active mode in agreement. The table must be initialized, aligned, readable and
/// valid for the returned borrow, without end-pointer overflow or conflicting mutable aliases.
unsafe fn table_from_window(_parent: &PageTableEntry, address: VirtAddrRT) -> &PageTable {
    unsafe { &*address.as_ptr::<PageTable>() }
}

/// Converts a recursive-window address into an exclusive child-table borrow bounded by `_parent`.
///
/// # Safety
///
/// The address and lifetime requirements of [`table_from_window`] apply. The child must also
/// be writable, and no other live reference may conflict with the returned exclusive borrow.
/// A newly allocated table must be valid as a `PageTable` before this reference is created;
/// its entries are cleared by the caller before they are used for traversal.
unsafe fn table_from_window_mut(
    _parent: &mut PageTableEntry,
    address: VirtAddrRT,
) -> &mut PageTable {
    unsafe { &mut *address.as_mut_ptr::<PageTable>() }
}

/// Selects the parent-entry checks required before following a recursive child window.
#[derive(Clone, Copy)]
enum ParentAccess {
    /// Requires a present, non-huge frame, as used by unmap, clear, and translation.
    Present,
    /// Rejects unused and huge entries, but does not require PRESENT, matching the baseline
    /// flag-update paths. An accepted entry alone does not prove accessibility.
    NonUnused,
}

impl ParentAccess {
    /// Validates the entry under this policy without dereferencing a window.
    ///
    /// Rejected entries are represented by [`WalkError::Frame`]. `NonUnused` maps an unused
    /// entry to `FrameNotPresent` even though it otherwise does not test the PRESENT flag.
    fn check(self, entry: &PageTableEntry) -> Result<(), WalkError> {
        match self {
            Self::Present => {
                entry.frame(false).map_err(WalkError::Frame)?;
            }
            Self::NonUnused => {
                if entry.is_unused() {
                    return Err(WalkError::Frame(FrameError::FrameNotPresent));
                }
                if entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                    return Err(WalkError::Frame(FrameError::HugeFrame));
                }
            }
        }
        Ok(())
    }
}

/// Central error conversion for all shared paths.
#[derive(Debug)]
enum WalkError {
    /// The target fails its own validity policy or cannot be represented by this root.
    AddressNotValid,
    /// A parent entry is absent/unused or describes a huge mapping instead of a child table.
    Frame(FrameError),
}

impl WalkError {
    /// Converts validation and parent-check failures to the errors used by unmap and clear.
    fn unmap(self) -> UnmapError {
        match self {
            Self::AddressNotValid => UnmapError::AddressNotValid,
            Self::Frame(FrameError::FrameNotPresent) => UnmapError::PageNotMapped,
            Self::Frame(FrameError::HugeFrame) => UnmapError::ParentEntryHugePage,
        }
    }

    /// Converts validation and parent-check failures to flag-update errors.
    fn flags(self) -> FlagUpdateError {
        match self {
            Self::AddressNotValid => FlagUpdateError::AddressNotValid,
            Self::Frame(FrameError::FrameNotPresent) => FlagUpdateError::PageNotMapped,
            Self::Frame(FrameError::HugeFrame) => FlagUpdateError::ParentEntryHugePage,
        }
    }

    /// Converts validation and parent-check failures to page-translation errors.
    fn translate(self) -> TranslateError {
        match self {
            Self::AddressNotValid => TranslateError::AddressNotValid,
            Self::Frame(FrameError::FrameNotPresent) => TranslateError::PageNotMapped,
            Self::Frame(FrameError::HugeFrame) => TranslateError::ParentEntryHugePage,
        }
    }
}

/// Returns the leaf table level: P3 for 1GiB, P2 for 2MiB, or P1 for 4KiB.
fn leaf_level<S: PageSize>() -> PageTableLevel {
    // PageSize is sealed and has exactly these three implementations.
    match S::SIZE {
        Size1GiB::SIZE => PageTableLevel::Three,
        Size2MiB::SIZE => PageTableLevel::Two,
        Size4KiB::SIZE => PageTableLevel::One,
        _ => unreachable!("PageSize is sealed to the three architectural page sizes"),
    }
}

/// Adds HUGE_PAGE for 1GiB/2MiB leaves and leaves the supplied 4KiB flags unchanged.
fn leaf_flags<S: PageSize>(flags: PageTableFlags) -> PageTableFlags {
    if leaf_level::<S>() == PageTableLevel::One {
        flags
    } else {
        flags | PageTableFlags::HUGE_PAGE
    }
}

/// Reads a leaf's frame address and checks alignment for `S`.
///
/// Misalignment returns `InvalidFrameAddress`. This does not check PRESENT or HUGE_PAGE;
/// callers perform those leaf checks before extracting the frame.
fn leaf_frame<S: PageSize>(entry: &PageTableEntry) -> Result<PhysFrame<S>, UnmapError> {
    PhysFrame::from_start_address(entry.addr())
        .map_err(|AddressNotAligned| UnmapError::InvalidFrameAddress(entry.addr()))
}

/// All three sizes share the path; only leaf level/flags and huge-leaf validation differ.
impl<S: PageSize, V: VirtAddrValidity> Mapper<S, V> for RecursivePageTable<'_> {
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<S, V>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<S, V>, MapToError<S>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let indices = self
            .validated_indices(page.start_address())
            .map_err(|_| MapToError::AddressNotValid)?;
        let level = leaf_level::<S>();
        let table =
            unsafe { self.create_table::<S, A>(&indices, level, parent_table_flags, allocator)? };
        let entry = &mut table[indices.index(level)];
        if !entry.is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        entry.set_addr(frame.start_address(), leaf_flags::<S>(flags));
        Ok(MapperFlush::new(page))
    }

    fn unmap(
        &mut self,
        page: Page<S, V>,
    ) -> Result<(PhysFrame<S>, PageTableFlags, MapperFlush<S, V>), UnmapError> {
        let indices = self
            .validated_indices(page.start_address())
            .map_err(WalkError::unmap)?;
        let level = leaf_level::<S>();
        let table = self
            .table_mut(&indices, level, ParentAccess::Present)
            .map_err(WalkError::unmap)?;
        let entry = &mut table[indices.index(level)];
        let flags = entry.flags();
        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if level != PageTableLevel::One && !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }
        let frame = leaf_frame(entry)?;
        entry.set_unused();
        Ok((frame, flags, MapperFlush::new(page)))
    }

    fn clear(&mut self, page: Page<S, V>) -> Result<UnmappedFrame<S, V>, UnmapError> {
        let indices = self
            .validated_indices(page.start_address())
            .map_err(WalkError::unmap)?;
        let level = leaf_level::<S>();
        let table = self
            .table_mut(&indices, level, ParentAccess::Present)
            .map_err(WalkError::unmap)?;
        let entry = &mut table[indices.index(level)];
        let flags = entry.flags();
        if level != PageTableLevel::One && !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }
        if !flags.contains(PageTableFlags::PRESENT) {
            let saved = entry.clone();
            entry.set_unused();
            return Ok(UnmappedFrame::NotPresent { entry: saved });
        }
        let frame = leaf_frame(entry)?;
        entry.set_unused();
        Ok(UnmappedFrame::Present {
            frame,
            flags,
            flush: MapperFlush::new(page),
        })
    }

    unsafe fn update_flags(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<S, V>, FlagUpdateError> {
        let indices = self
            .validated_indices(page.start_address())
            .map_err(WalkError::flags)?;
        let level = leaf_level::<S>();
        let table = self
            .table_mut(&indices, level, ParentAccess::NonUnused)
            .map_err(WalkError::flags)?;
        let entry = &mut table[indices.index(level)];
        if entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        entry.set_flags(leaf_flags::<S>(flags));
        Ok(MapperFlush::new(page))
    }

    unsafe fn set_flags_p5_entry(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        if self.root_level == PageTableRootLevel::Four {
            return Err(FlagUpdateError::PageTableLevelNotPresent);
        }
        self.set_parent_flags(page, PageTableLevel::Five, flags)
    }

    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.set_parent_flags(page, PageTableLevel::Four, flags)
    }

    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.set_parent_flags(page, PageTableLevel::Three, flags)
    }

    unsafe fn set_flags_p2_entry(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.set_parent_flags(page, PageTableLevel::Two, flags)
    }

    fn translate_page(&self, page: Page<S, V>) -> Result<PhysFrame<S>, TranslateError> {
        let indices = self
            .validated_indices(page.start_address())
            .map_err(WalkError::translate)?;
        let level = leaf_level::<S>();
        let table = self
            .table(&indices, level, ParentAccess::Present)
            .map_err(WalkError::translate)?;
        let entry = &table[indices.index(level)];
        if !entry.flags().contains(PageTableFlags::PRESENT)
            || (level != PageTableLevel::One && !entry.flags().contains(PageTableFlags::HUGE_PAGE))
        {
            return Err(TranslateError::PageNotMapped);
        }
        PhysFrame::from_start_address(entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(entry.addr()))
    }
}

impl<V: VirtAddrValidity> Translate<V> for RecursivePageTable<'_> {
    fn translate(&self, address: VirtAddrGeneric<V>) -> TranslateResult {
        let indices = match self.validated_indices(address) {
            Ok(indices) => indices,
            Err(_) => return TranslateResult::AddressNotValid,
        };
        let mut table = &*self.root_table;
        let mut current_level = self.root_level.as_page_table_level();
        loop {
            let entry = &table[indices.index(current_level)];
            if !entry.flags().contains(PageTableFlags::PRESENT) {
                return TranslateResult::NotMapped;
            }
            if current_level == PageTableLevel::One {
                let frame = match PhysFrame::from_start_address(entry.addr()) {
                    Ok(frame) => frame,
                    Err(AddressNotAligned) => {
                        return TranslateResult::InvalidFrameAddress(entry.addr());
                    }
                };
                return TranslateResult::Mapped {
                    frame: MappedFrame::Size4KiB(frame),
                    offset: u64::from(address.page_offset()),
                    flags: entry.flags(),
                };
            }
            if entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                let (frame, offset) = match current_level {
                    PageTableLevel::Three => (
                        MappedFrame::Size1GiB(PhysFrame::containing_address(entry.addr())),
                        address.as_u64() % Size1GiB::SIZE,
                    ),
                    PageTableLevel::Two => (
                        MappedFrame::Size2MiB(PhysFrame::containing_address(entry.addr())),
                        address.as_u64() % Size2MiB::SIZE,
                    ),
                    PageTableLevel::Four => panic!("level 4 entry has huge page bit set"),
                    PageTableLevel::Five => panic!("level 5 entry has huge page bit set"),
                    PageTableLevel::One => unreachable!("level 1 was handled above"),
                };
                return TranslateResult::Mapped {
                    frame,
                    offset,
                    flags: entry.flags(),
                };
            }
            let child_level = current_level
                .next_lower_level()
                .expect("a non-leaf entry has a child");
            table = match Self::next_table(
                self.root_level,
                self.recursive_index,
                entry,
                &indices,
                child_level,
                ParentAccess::Present,
            ) {
                Ok(table) => table,
                Err(WalkError::Frame(FrameError::FrameNotPresent)) => {
                    return TranslateResult::NotMapped;
                }
                Err(WalkError::Frame(FrameError::HugeFrame)) => {
                    unreachable!("non-leaf huge entries were handled above")
                }
                Err(WalkError::AddressNotValid) => {
                    unreachable!("the target address was validated before traversal")
                }
            };
            current_level = child_level;
        }
    }
}

impl CleanUp for RecursivePageTable<'_> {
    unsafe fn clean_up<D>(&mut self, frame_deallocator: &mut D)
    where
        D: FrameDeallocator<Size4KiB>,
    {
        // A single range covers both canonical halves; the walker skips the canonical hole.
        let end = u64::MAX - (Size4KiB::SIZE - 1);
        match self.root_level {
            PageTableRootLevel::Four => unsafe {
                self.clean_up_range::<FixedValidity<48>, D>(0, end, frame_deallocator)
            },
            PageTableRootLevel::Five => unsafe {
                self.clean_up_range::<RuntimeValidity, D>(0, end, frame_deallocator)
            },
        }
    }

    unsafe fn clean_up_addr_range<D, V: VirtAddrValidity>(
        &mut self,
        range: PageRangeInclusive<Size4KiB, V>,
        frame_deallocator: &mut D,
    ) where
        D: FrameDeallocator<Size4KiB>,
    {
        // Invalid endpoints violate the public unsafe precondition, rather than being an
        // ordinary recoverable error. Use the same target/root checks as Mapper and Translate.
        self.validated_indices(range.start.start_address())
            .expect("cleanup start must be valid for the target policy and root");
        self.validated_indices(range.end.start_address())
            .expect("cleanup end must be valid for the target policy and root");
        if range.is_empty() {
            return;
        }
        let start = range.start.start_address().as_u64();
        let end = range.end.start_address().as_u64();
        match self.root_level {
            PageTableRootLevel::Four => unsafe {
                self.clean_up_range::<FixedValidity<48>, D>(start, end, frame_deallocator)
            },
            PageTableRootLevel::Five => unsafe {
                self.clean_up_range::<RuntimeValidity, D>(start, end, frame_deallocator)
            },
        }
    }
}

/// A cleanup interval cannot be computed within the root's canonical address space.
#[derive(Debug)]
struct CleanupAddressError;

/// Computes the inclusive 4KiB-page range covered by an entry at `level`.
///
/// `table_address` is the table's logical coverage base, not its recursive-window address;
/// it is zero for the root. `R` must describe the root's canonical space, and `index` must
/// identify an entry in the table. Forwarding skips the canonical hole. Overflow or a range
/// that cannot be represented under `R` returns [`CleanupAddressError`].
fn cleanup_entry_range<R: VirtAddrValidity>(
    table_address: VirtAddrGeneric<R>,
    level: PageTableLevel,
    index: usize,
) -> Result<PageRangeInclusive<Size4KiB, R>, CleanupAddressError> {
    let span = level.entry_address_space_alignment();
    let offset = span.checked_mul(index as u64).ok_or(CleanupAddressError)?;
    let start = cleanup_entry_address(table_address, offset).ok_or(CleanupAddressError)?;
    let end = cleanup_entry_address(start, span - 1).ok_or(CleanupAddressError)?;
    Ok(Page::range_inclusive(
        Page::containing_address(start),
        Page::containing_address(end),
    ))
}

/// Visits child tables overlapping `range`, clears empty-child entries and frees their frames.
///
/// Skips only the root's recursive entry, along with entries that do not yield a present,
/// non-huge child frame. Leaves and the supplied table itself are never freed by this call.
/// Returns whether the entire supplied table is empty, not merely its intersection with
/// `range`. Address errors propagate separately from that boolean; earlier cleanup is not
/// rolled back if a later entry fails.
///
/// # Safety
///
/// `table`, `level` and `table_address` must describe the same table in the active hierarchy
/// selected by `root_level` and `recursive_index`. The logical root base must be zero; child
/// bases must describe their full entry coverage. `R` must match the root's traversal width,
/// and `range` must be a valid nonempty intersection with this table's coverage.
/// Recursive windows must permit exclusive child
/// access. Freed frames must have no other aliases or uses, as required by [`CleanUp`].
unsafe fn clean_up_table<R: VirtAddrValidity, D: FrameDeallocator<Size4KiB>>(
    table: &mut PageTable,
    root_level: PageTableRootLevel,
    recursive_index: PageTableIndex,
    level: PageTableLevel,
    table_address: VirtAddrGeneric<R>,
    range: PageRangeInclusive<Size4KiB, R>,
    deallocator: &mut D,
) -> Result<bool, CleanupAddressError> {
    if let Some(child_level) = level.next_lower_level() {
        for (index, entry) in table.iter_mut().enumerate() {
            if RecursivePageTable::is_root_recursive_entry(
                root_level,
                recursive_index,
                level,
                index,
            ) {
                continue;
            }
            let Ok(frame) = entry.frame(false) else {
                // Absent entries and huge leaves do not own an accessible child table.
                continue;
            };
            let entry_range = cleanup_entry_range(table_address, level, index)?;
            let start = entry_range.start.max(range.start);
            let end = entry_range.end.min(range.end);
            if start > end {
                continue;
            }
            let indices =
                PageTableIndices::from_address(entry_range.start.start_address().as_u64());
            // SAFETY: this present ordinary entry maps the child identified by indices and
            // child_level. The cleanup caller guarantees exclusive, unshared frame ownership.
            let child = unsafe {
                table_from_window_mut(
                    entry,
                    RecursivePageTable::recursive_window_address(
                        root_level,
                        recursive_index,
                        &indices,
                        child_level,
                    ),
                )
            };
            if unsafe {
                clean_up_table(
                    child,
                    root_level,
                    recursive_index,
                    child_level,
                    entry_range.start.start_address(),
                    Page::range_inclusive(start, end),
                    deallocator,
                )?
            } {
                entry.set_unused();
                unsafe { deallocator.deallocate_frame(frame) };
            }
        }
    }
    Ok(table.is_empty())
}

/// The supplied table cannot be used as an active recursively mapped root.
#[derive(Debug, PartialEq, Eq)]
pub enum InvalidPageTable {
    /// The table address does not repeat its recursive index at every active level.
    NotRecursive,
    /// The explicitly requested root level differs from the active CR4.LA57 mode.
    ModeMismatch,
    /// The recursive index is 511, which cannot safely back a `PageTable` reference.
    RecursiveIndexUnavailable,
    /// The table address is invalid in the current mode, or its recursive entry does not point
    /// to the active CR3 frame.
    NotActive,
}

impl fmt::Display for InvalidPageTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRecursive => write!(f, "given page table address is not recursive"),
            Self::ModeMismatch => write!(
                f,
                "requested root level differs from the active paging mode"
            ),
            Self::RecursiveIndexUnavailable => {
                write!(
                    f,
                    "recursive index 511 cannot safely back a page table reference"
                )
            }
            Self::NotActive => write!(f, "given page table is not active on the CPU"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PhysAddr;
    use crate::addr::{canonicalize_with_bits, forward_checked_with_bits};
    use std::vec::Vec;

    /// Builds a child window from raw target bits and an explicit canonical width for host tests.
    ///
    /// This exercises the production offset builder without initializing the LA57 cache.
    /// The requested table must be below the root and produce a representable window.
    fn window_address(
        address: u64,
        bits: usize,
        root: PageTableRootLevel,
        table: PageTableLevel,
        recursive_index: u16,
    ) -> u64 {
        let indices = PageTableIndices::from_address(address);
        let raw = RecursivePageTable::recursive_window_offset(
            root,
            PageTableIndex::new(recursive_index),
            &indices,
            table,
        )
        .unwrap();
        canonicalize_with_bits(raw, bits)
    }

    /// Packs four or five root-to-leaf indices into a canonical address of the supplied width.
    ///
    /// Test fixtures supply valid entry indices. This helper does not use the recursive-window
    /// builder, so expected index sequences independently specify each window's layout.
    fn expected_address(indices: &[u16], bits: usize) -> u64 {
        let levels = match indices.len() {
            4 => &[
                PageTableLevel::Four,
                PageTableLevel::Three,
                PageTableLevel::Two,
                PageTableLevel::One,
            ][..],
            5 => &[
                PageTableLevel::Five,
                PageTableLevel::Four,
                PageTableLevel::Three,
                PageTableLevel::Two,
                PageTableLevel::One,
            ][..],
            _ => panic!("only architectural roots are supported"),
        };
        let mut raw = 0u64;
        for (index, level) in indices.iter().zip(levels) {
            raw += u64::from(*index) * level.entry_address_space_alignment();
        }
        canonicalize_with_bits(raw, bits)
    }

    /// Lists an address's indices from the selected root down to P1 without consulting CR4.
    fn indices(address: u64, root: PageTableRootLevel) -> Vec<u16> {
        let indices = PageTableIndices::from_address(address);
        let mut result = Vec::new();
        let mut level = Some(root.as_page_table_level());
        while let Some(current) = level {
            result.push(u16::from(indices.index(current)));
            level = current.next_lower_level();
        }
        result
    }

    /// Checks a window's index sequence, packed address and canonicality against a fixture.
    fn assert_window(
        address: u64,
        bits: usize,
        root: PageTableRootLevel,
        table: PageTableLevel,
        r: u16,
        expected: &[u16],
    ) {
        let actual = window_address(address, bits, root, table, r);
        assert_eq!(indices(actual, root), expected);
        assert_eq!(actual, expected_address(expected, bits));
        assert_eq!(canonicalize_with_bits(actual, bits), actual);
    }

    /// Checks that recursive shape validation ignores indices above the explicitly chosen root.
    #[test]
    fn recursive_shape_uses_the_explicit_root_not_all_address_indices() {
        let address = expected_address(&[2, 1, 1, 1, 1], 57);
        let indices = PageTableIndices::from_address(address);
        assert_eq!(
            indices.recursive_index(PageTableRootLevel::Four),
            Some(PageTableIndex::new(1))
        );
        assert_eq!(indices.recursive_index(PageTableRootLevel::Five), None);
        assert_eq!(indices.index(PageTableLevel::Five), PageTableIndex::new(2));
    }

    /// Checks that all three shared paths start at the mapper's root and reject huge parents.
    #[test]
    fn shared_paths_start_at_the_mappers_root_level() {
        /// Fails if a path allocates before recognizing the fixture's existing huge root entry.
        struct UnexpectedAllocator;
        unsafe impl FrameAllocator<Size4KiB> for UnexpectedAllocator {
            /// Rejects allocations, which are never needed for this fixture.
            fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
                panic!("the existing root entry must be checked before allocation");
            }
        }

        // Distinct P5/P4 indices reveal traversals that use the wrong starting level.
        let indices = PageTableIndices::from_address(expected_address(&[2, 1, 3, 4, 5], 57));
        for root_level in [PageTableRootLevel::Four, PageTableRootLevel::Five] {
            let root_index = match root_level {
                PageTableRootLevel::Four => PageTableIndex::new(1),
                PageTableRootLevel::Five => PageTableIndex::new(2),
            };
            let mut root = PageTable::new();
            root[root_index].set_addr(
                PhysAddr::new(0),
                PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE,
            );
            // The root access and rejected descent below never build a window or read CR4.
            let mut mapper = RecursivePageTable {
                root_table: &mut root,
                recursive_index: PageTableIndex::new(510),
                root_level,
            };
            let root_ptr = mapper.root_table() as *const PageTable;
            let level = root_level.as_page_table_level();
            assert!(core::ptr::eq(
                mapper
                    .table(&indices, level, ParentAccess::Present)
                    .unwrap(),
                root_ptr,
            ));
            assert!(core::ptr::eq(
                mapper
                    .table_mut(&indices, level, ParentAccess::Present)
                    .unwrap(),
                root_ptr,
            ));
            assert!(core::ptr::eq(
                unsafe {
                    mapper.create_table::<Size4KiB, _>(
                        &indices,
                        level,
                        PageTableFlags::empty(),
                        &mut UnexpectedAllocator,
                    )
                }
                .unwrap(),
                root_ptr,
            ));
            assert!(matches!(
                mapper.table(&indices, PageTableLevel::One, ParentAccess::Present),
                Err(WalkError::Frame(FrameError::HugeFrame))
            ));
            assert!(matches!(
                mapper.table_mut(&indices, PageTableLevel::One, ParentAccess::Present),
                Err(WalkError::Frame(FrameError::HugeFrame))
            ));
            assert!(matches!(
                unsafe {
                    mapper.create_table::<Size4KiB, _>(
                        &indices,
                        PageTableLevel::One,
                        PageTableFlags::empty(),
                        &mut UnexpectedAllocator,
                    )
                },
                Err(MapToError::ParentEntryHugePage)
            ));
        }
    }

    /// Covers each L4 child window in both canonical halves and at recursive-index boundaries.
    #[test]
    fn l4_windows_cover_every_child_halves_and_recursive_index_boundaries() {
        for address in [0x0000_1234_5678_9000, 0xffff_9234_5678_9000] {
            let indices = PageTableIndices::from_address(address);
            let a4 = u16::from(indices.index(PageTableLevel::Four));
            let a3 = u16::from(indices.index(PageTableLevel::Three));
            let a2 = u16::from(indices.index(PageTableLevel::Two));
            for r in [0, 1, 255, 256, 510, 511] {
                assert_window(
                    address,
                    48,
                    PageTableRootLevel::Four,
                    PageTableLevel::Three,
                    r,
                    &[r, r, r, a4],
                );
                assert_window(
                    address,
                    48,
                    PageTableRootLevel::Four,
                    PageTableLevel::Two,
                    r,
                    &[r, r, a4, a3],
                );
                assert_window(
                    address,
                    48,
                    PageTableRootLevel::Four,
                    PageTableLevel::One,
                    r,
                    &[r, a4, a3, a2],
                );
            }
        }
    }

    /// Covers each L5 child window, including ordinary treatment of VA48 P5 indices zero and 511.
    #[test]
    fn l5_windows_cover_every_child_and_all_kinds_of_p5_indices() {
        for address in [
            0x0000_1234_5678_9000, // VA48 lower half, P5 = 0
            0xffff_9234_5678_9000, // VA48 upper half, P5 = 511
            0x0056_1234_5678_9000, // VA57 lower half
            0xff56_1234_5678_9000, // VA57 upper half
        ] {
            let indices = PageTableIndices::from_address(address);
            let a5 = u16::from(indices.index(PageTableLevel::Five));
            let a4 = u16::from(indices.index(PageTableLevel::Four));
            let a3 = u16::from(indices.index(PageTableLevel::Three));
            let a2 = u16::from(indices.index(PageTableLevel::Two));
            for r in [0, 1, 255, 256, 510, 511] {
                assert_window(
                    address,
                    57,
                    PageTableRootLevel::Five,
                    PageTableLevel::Four,
                    r,
                    &[r, r, r, r, a5],
                );
                assert_window(
                    address,
                    57,
                    PageTableRootLevel::Five,
                    PageTableLevel::Three,
                    r,
                    &[r, r, r, a5, a4],
                );
                assert_window(
                    address,
                    57,
                    PageTableRootLevel::Five,
                    PageTableLevel::Two,
                    r,
                    &[r, r, a5, a4, a3],
                );
                assert_window(
                    address,
                    57,
                    PageTableRootLevel::Five,
                    PageTableLevel::One,
                    r,
                    &[r, a5, a4, a3, a2],
                );
            }
        }
    }

    /// Checks repeated-index root shapes and rejects mismatches without dereferencing index 511.
    #[test]
    fn recursive_shape_checks_all_active_indices_and_keeps_index_511() {
        for r in [0, 1, 255, 256, 510, 511] {
            for (root, bits, count) in [
                (PageTableRootLevel::Four, 48, 4),
                (PageTableRootLevel::Five, 57, 5),
            ] {
                let repeated = [r; 5];
                let address = expected_address(&repeated[..count], bits);
                let indices = PageTableIndices::from_address(address);
                assert_eq!(indices.recursive_index(root), Some(PageTableIndex::new(r)));
                assert!(
                    RecursivePageTable::recursive_window_offset(
                        root,
                        PageTableIndex::new(r),
                        &indices,
                        root.as_page_table_level(),
                    )
                    .is_none()
                );
                let mut malformed = indices;
                malformed.set_index(PageTableLevel::One, PageTableIndex::new_truncate(r + 1));
                assert_eq!(malformed.recursive_index(root), None);
            }
        }
    }

    #[test]
    fn recursive_address_errors_distinguish_mode_and_index_failures() {
        let valid = expected_address(&[7, 7, 7, 7], 48);
        assert_eq!(
            classify_recursive_address(valid, PageTableRootLevel::Four, 48),
            Ok(PageTableIndex::new(7))
        );

        let mode_mismatch = expected_address(&[1, 1, 1, 1, 1], 57);
        assert_eq!(
            classify_recursive_address(mode_mismatch, PageTableRootLevel::Five, 48),
            Err(InvalidPageTable::ModeMismatch)
        );

        let not_recursive = expected_address(&[7, 6, 7, 7], 48);
        assert_eq!(
            classify_recursive_address(not_recursive, PageTableRootLevel::Four, 48),
            Err(InvalidPageTable::NotRecursive)
        );

        let unavailable = expected_address(&[511, 511, 511, 511], 48);
        assert_eq!(
            classify_recursive_address(unavailable, PageTableRootLevel::Four, 48),
            Err(InvalidPageTable::RecursiveIndexUnavailable)
        );
    }

    #[test]
    fn recursive_address_checks_cover_both_modes_and_index_boundaries() {
        for (root, bits, count) in [
            (PageTableRootLevel::Four, 48, 4),
            (PageTableRootLevel::Five, 57, 5),
        ] {
            for r in [0, 1, 255, 256, 510, 511] {
                let repeated = [r; 5];
                let address = expected_address(&repeated[..count], bits);
                let expected = if r == 511 {
                    Err(InvalidPageTable::RecursiveIndexUnavailable)
                } else {
                    Ok(PageTableIndex::new(r))
                };
                assert_eq!(classify_recursive_address(address, root, bits), expected);
                // Flip a sign-extension bit without changing the repeated page-table indices.
                assert_eq!(
                    classify_recursive_address(address ^ (1 << bits), root, bits),
                    Err(InvalidPageTable::NotActive),
                );
            }
        }
    }

    #[test]
    fn recursive_mode_check_precedes_address_shape_checks() {
        // Zero has a repeated-index shape in both modes, so shape cannot detect this mismatch.
        assert_eq!(
            classify_recursive_address(0, PageTableRootLevel::Four, 57),
            Err(InvalidPageTable::ModeMismatch)
        );
        assert_eq!(
            classify_recursive_address(0, PageTableRootLevel::Five, 48),
            Err(InvalidPageTable::ModeMismatch)
        );
    }

    #[test]
    fn recursive_shape_failure_does_not_imply_a_different_root_mode() {
        // P1-P4 repeat, but P5 differs. The caller explicitly requested the active L5 mode.
        let address = expected_address(&[2, 7, 7, 7, 7], 57);
        assert_eq!(
            classify_recursive_address(address, PageTableRootLevel::Five, 57),
            Err(InvalidPageTable::NotRecursive)
        );
    }

    /// Verifies the runtime window type and L5 width independently of a fixed-VA48 target policy.
    #[test]
    fn fixed_target_policy_is_independent_of_the_runtime_window_type_and_width() {
        // A type assertion, without invoking the method or reading CR4.
        let _: fn(
            PageTableRootLevel,
            PageTableIndex,
            &PageTableIndices,
            PageTableLevel,
        ) -> VirtAddrRT = RecursivePageTable::recursive_window_address;
        let _: unsafe fn(
            &mut RecursivePageTable<'static>,
            PageRangeInclusive<Size4KiB, RuntimeValidity>,
            &mut NoopDeallocator,
        ) = <RecursivePageTable<'static> as CleanUp>::clean_up_addr_range::<
            NoopDeallocator,
            RuntimeValidity,
        >;

        for (address, p5) in [(0x1234_5000, 0), (0xffff_8000_1234_5000, 511)] {
            let target =
                Page::<Size4KiB, FixedValidity<48>>::containing_address(VirtAddrGeneric::<
                    FixedValidity<48>,
                >::new(
                    address
                ));
            let target_indices = PageTableIndices::from_address(target.start_address().as_u64());
            assert_eq!(u16::from(target_indices.index(PageTableLevel::Five)), p5);
            let raw = RecursivePageTable::recursive_window_offset(
                PageTableRootLevel::Five,
                PageTableIndex::new(1),
                &target_indices,
                PageTableLevel::One,
            )
            .unwrap();
            let window = window_address(
                address,
                57,
                PageTableRootLevel::Five,
                PageTableLevel::One,
                1,
            );
            assert_eq!(window, canonicalize_with_bits(raw, 57));
            assert_ne!(window, canonicalize_with_bits(raw, 48));
            assert_eq!(indices(window, PageTableRootLevel::Five)[1], p5);
        }
    }

    /// A deallocator used only to instantiate the cleanup method's compile-time type assertion.
    struct NoopDeallocator;

    impl FrameDeallocator<Size4KiB> for NoopDeallocator {
        /// Does nothing; no frame is actually released by the type assertion.
        unsafe fn deallocate_frame(&mut self, _frame: PhysFrame<Size4KiB>) {}
    }

    /// Checks the L4 logical root base, canonical-hole forwarding and address-error propagation.
    #[test]
    fn l4_cleanup_uses_a_zero_root_base_and_forwards_across_the_hole() {
        type V48 = FixedValidity<48>;
        let root = PageTableLevel::Four;
        let upper = V48::upper_half_start();
        let zero = cleanup_table_address(root, root, VirtAddrGeneric::<V48>::new(upper));
        assert_eq!(zero.as_u64(), 0);
        let first_upper_index =
            root.table_address_space_alignment() / 2 / root.entry_address_space_alignment();
        let range = cleanup_entry_range(zero, root, first_upper_index as usize).unwrap();
        assert_eq!(range.start.start_address().as_u64(), upper);
        let lower_end = VirtAddrGeneric::<V48>::new(V48::lower_half_end());
        let forwarded = VirtAddrGeneric::<V48>::forward_checked_u64(lower_end, 1).unwrap();
        assert_eq!(forwarded.as_u64(), upper);
        assert_eq!(
            forward_checked_with_bits(lower_end.as_u64(), 1, 48),
            Some(upper)
        );
        assert!(forward_checked_with_bits(u64::MAX, 1, 48).is_none());

        // Address failures are propagated, rather than looking like a nonempty child table.
        assert!(cleanup_entry_range(zero, PageTableLevel::Five, 1).is_err());
    }

    /// Checks canonical forwarding for both roots and recursive-entry skipping only at the root.
    #[test]
    fn canonical_forwarding_and_recursive_entry_skipping_match_each_root() {
        for (root_level, bits) in [
            (PageTableRootLevel::Four, 48),
            (PageTableRootLevel::Five, 57),
        ] {
            let root = root_level.as_page_table_level();
            let half_space = root.table_address_space_alignment() / 2;
            let first_upper = canonicalize_with_bits(half_space, bits);
            assert_eq!(
                forward_checked_with_bits(0, half_space, bits),
                Some(first_upper)
            );
            assert_eq!(
                forward_checked_with_bits(half_space - 1, 1, bits),
                Some(first_upper)
            );
            for recursive_index in [0, 1, 510, 511] {
                for level in [
                    PageTableLevel::One,
                    PageTableLevel::Two,
                    PageTableLevel::Three,
                    PageTableLevel::Four,
                    PageTableLevel::Five,
                ] {
                    assert_eq!(
                        RecursivePageTable::is_root_recursive_entry(
                            root_level,
                            PageTableIndex::new(recursive_index),
                            level,
                            recursive_index as usize,
                        ),
                        level == root,
                    );
                    assert!(!RecursivePageTable::is_root_recursive_entry(
                        root_level,
                        PageTableIndex::new(recursive_index),
                        level,
                        usize::from(PageTableIndex::new_truncate(recursive_index + 1)),
                    ));
                }
            }
        }
    }

    /// Checks L5 root-entry coverage and forwarding across the VA57 canonical boundary.
    #[cfg(feature = "virt_addr_57")]
    #[test]
    fn l5_cleanup_uses_its_canonical_boundary_and_supports_a_single_full_range() {
        type V57 = FixedValidity<57>;
        let root = PageTableLevel::Five;
        let upper = V57::upper_half_start();
        let zero = cleanup_table_address(root, root, VirtAddrGeneric::<V57>::new(upper));
        assert_eq!(zero.as_u64(), 0);
        let entries = root.table_address_space_alignment() / root.entry_address_space_alignment();
        let first_upper = cleanup_entry_range(zero, root, (entries / 2) as usize).unwrap();
        assert_eq!(first_upper.start.start_address().as_u64(), upper);
        let last = cleanup_entry_range(zero, root, entries as usize - 1).unwrap();
        assert_eq!(
            last.end.start_address().as_u64(),
            u64::MAX - (Size4KiB::SIZE - 1)
        );
        let lower_end = VirtAddrGeneric::<V57>::new(V57::lower_half_end());
        assert_eq!(
            VirtAddrGeneric::<V57>::forward_checked_u64(lower_end, 1)
                .unwrap()
                .as_u64(),
            upper
        );
        assert_eq!(
            forward_checked_with_bits(lower_end.as_u64(), 1, 57),
            Some(upper)
        );
        assert!(forward_checked_with_bits(u64::MAX, 1, 57).is_none());
    }

    /// Checks parent-access policies, public error conversions and size-specific leaf handling.
    #[test]
    fn shared_parent_checks_and_error_conversions_preserve_baseline_semantics() {
        let mut entry = PageTableEntry::new();
        assert!(matches!(
            ParentAccess::Present.check(&entry),
            Err(WalkError::Frame(FrameError::FrameNotPresent))
        ));
        entry.set_addr(PhysAddr::new(0), PageTableFlags::USER_ACCESSIBLE);
        assert!(ParentAccess::NonUnused.check(&entry).is_ok());
        assert!(matches!(
            ParentAccess::Present.check(&entry),
            Err(WalkError::Frame(FrameError::FrameNotPresent))
        ));
        entry.set_flags(PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE);
        for access in [ParentAccess::Present, ParentAccess::NonUnused] {
            assert!(matches!(
                access.check(&entry),
                Err(WalkError::Frame(FrameError::HugeFrame))
            ));
        }
        assert!(matches!(
            WalkError::Frame(FrameError::HugeFrame).unmap(),
            UnmapError::ParentEntryHugePage
        ));
        assert!(matches!(
            WalkError::Frame(FrameError::HugeFrame).flags(),
            FlagUpdateError::ParentEntryHugePage
        ));
        assert!(matches!(
            WalkError::Frame(FrameError::HugeFrame).translate(),
            TranslateError::ParentEntryHugePage
        ));
        assert_eq!(leaf_level::<Size1GiB>(), PageTableLevel::Three);
        assert_eq!(leaf_level::<Size2MiB>(), PageTableLevel::Two);
        assert_eq!(leaf_level::<Size4KiB>(), PageTableLevel::One);
        assert_eq!(
            leaf_flags::<Size4KiB>(PageTableFlags::empty()),
            PageTableFlags::empty()
        );
        assert!(
            leaf_flags::<Size1GiB>(PageTableFlags::empty()).contains(PageTableFlags::HUGE_PAGE)
        );
        assert!(
            leaf_flags::<Size2MiB>(PageTableFlags::empty()).contains(PageTableFlags::HUGE_PAGE)
        );
    }

    #[test]
    fn translation_rejects_non_present_parent_before_window_access() {
        for flags in [
            PageTableFlags::BIT_9,
            PageTableFlags::BIT_9 | PageTableFlags::HUGE_PAGE,
        ] {
            let mut root = PageTable::new();
            root[0].set_addr(PhysAddr::new(0x1000), flags);
            // Translation must stop at this root entry, before constructing or
            // dereferencing a recursive window or reading the runtime CR4 cache.
            let mapper = RecursivePageTable {
                root_table: &mut root,
                recursive_index: PageTableIndex::new(510),
                root_level: PageTableRootLevel::Four,
            };
            let address = VirtAddr48::new(0x123);
            assert!(matches!(
                mapper.translate(address),
                TranslateResult::NotMapped
            ));
            assert_eq!(mapper.translate_addr(address), None);
            assert!(matches!(
                mapper.translate_page(Page::<Size4KiB>::containing_address(address)),
                Err(TranslateError::PageNotMapped)
            ));
            assert!(matches!(
                mapper.translate_page(Page::<Size2MiB>::containing_address(address)),
                Err(TranslateError::PageNotMapped)
            ));
            assert!(matches!(
                mapper.translate_page(Page::<Size1GiB>::containing_address(address)),
                Err(TranslateError::PageNotMapped)
            ));
        }
    }

    /// Checks early rejection of VA57-only targets by L4 mapping and translation entry points.
    #[cfg(feature = "virt_addr_57")]
    #[test]
    fn l4_rejects_unrepresentable_fixed_targets_at_every_entry_point() {
        type V57 = FixedValidity<57>;
        /// Supplies no frames; invalid targets must be rejected before allocation matters.
        struct EmptyAllocator;
        unsafe impl FrameAllocator<Size4KiB> for EmptyAllocator {
            /// Reports that no physical frame is available.
            fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
                None
            }
        }
        // These operations fail before any window access; no constructor or register read.
        let mut root = PageTable::new();
        let mut mapper = RecursivePageTable {
            root_table: &mut root,
            recursive_index: PageTableIndex::new(1),
            root_level: PageTableRootLevel::Four,
        };
        let address = VirtAddrGeneric::<V57>::new(0x0001_0000_0000_0000);
        let page = Page::<Size4KiB, V57>::containing_address(address);
        assert!(matches!(
            mapper.translate(address),
            TranslateResult::AddressNotValid
        ));
        assert!(matches!(
            mapper.translate_page(page),
            Err(TranslateError::AddressNotValid)
        ));
        assert!(matches!(
            mapper.unmap(page),
            Err(UnmapError::AddressNotValid)
        ));
        assert!(matches!(
            mapper.clear(page),
            Err(UnmapError::AddressNotValid)
        ));
        assert!(matches!(
            unsafe {
                mapper.map_to(
                    page,
                    PhysFrame::from_start_address(PhysAddr::new(0)).unwrap(),
                    PageTableFlags::PRESENT,
                    &mut EmptyAllocator,
                )
            },
            Err(MapToError::AddressNotValid)
        ));
        assert!(matches!(
            unsafe { mapper.update_flags(page, PageTableFlags::PRESENT) },
            Err(FlagUpdateError::AddressNotValid)
        ));
        for level in [
            PageTableLevel::Five,
            PageTableLevel::Four,
            PageTableLevel::Three,
            PageTableLevel::Two,
        ] {
            assert!(matches!(
                mapper.set_parent_flags(page, level, PageTableFlags::PRESENT),
                Err(FlagUpdateError::AddressNotValid)
            ));
        }
        let valid = Page::<Size4KiB>::containing_address(crate::VirtAddr::zero());
        assert!(matches!(
            unsafe { mapper.set_flags_p5_entry(valid, PageTableFlags::PRESENT) },
            Err(FlagUpdateError::PageTableLevelNotPresent)
        ));
    }
}
