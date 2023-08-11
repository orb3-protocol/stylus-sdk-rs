// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use alloy_primitives::{Address, BlockHash, BlockNumber, FixedBytes, Signed, Uint, U256};
use std::{cell::OnceCell, ops::Deref};

pub use bytes::{StorageBytes, StorageString};
pub use cache::{
    Erasable, SimpleStorageType, StorageCache, StorageGuard, StorageGuardMut, StorageType,
};
pub use map::StorageMap;
pub use vec::StorageVec;

mod bytes;
mod cache;
mod map;
mod vec;

/// Overwrites the value in a cell.
#[inline]
fn overwrite_cell<T>(cell: &mut OnceCell<T>, value: T) {
    cell.take();
    _ = cell.set(value);
}

macro_rules! alias_ints {
    ($($name:ident, $signed_name:ident, $bits:expr, $limbs:expr;)*) => {
        $(
            #[doc = concat!("Accessor for a storage-backed [`U", stringify!($bits), "`].")]
            pub type $name = StorageUint<$bits, $limbs>;

            #[doc = concat!("Accessor for a storage-backed [`I", stringify!($bits), "`].")]
            pub type $signed_name = StorageSigned<$bits, $limbs>;
        )*
    };
}

macro_rules! alias_bytes {
    ($($name:ident, $bytes:expr;)*) => {
        $(
            #[doc = concat!("Accessor for a storage-backed [`B", stringify!($bytes), "`].")]
            pub type $name = StorageFixedBytes<$bytes>;
        )*
    };
}

alias_ints! {
    StorageU0, StorageI0, 0, 0;
    StorageU1, StorageI1, 1, 1;
    StorageU8, StorageI8, 8, 1;
    StorageU16, StorageI16, 16, 1;
    StorageU32, StorageI32, 32, 1;
    StorageU64, StorageI64, 64, 1;
    StorageU128, StorageI128, 128, 2;
    StorageU160, StorageI160, 160, 3;
    StorageU192, StorageI192, 192, 3;
    StorageU256, StorageI256, 256, 4;
}

alias_bytes! {
    StorageB0, 0;
    StorageB8, 1;
    StorageB16, 2;
    StorageB32, 4;
    StorageB64, 8;
    StorageB96, 12;
    StorageB128, 16;
    StorageB160, 20;
    StorageB192, 24;
    StorageB224, 28;
    StorageB256, 32;
}

/// Accessor for a storage-backed [`Uint`].
#[derive(Debug)]
pub struct StorageUint<const B: usize, const L: usize> {
    slot: U256,
    offset: u8,
    cached: OnceCell<Uint<B, L>>,
}

impl<const B: usize, const L: usize> StorageUint<B, L> {
    /// Gets the underlying [`Uint`] in persistent storage.
    pub fn get(&self) -> Uint<B, L> {
        **self
    }

    /// Sets the underlying [`Uint`] in persistent storage.
    pub fn set(&mut self, value: Uint<B, L>) {
        overwrite_cell(&mut self.cached, value);
        unsafe { StorageCache::set_uint(self.slot, self.offset.into(), value) };
    }
}

impl<const B: usize, const L: usize> StorageType for StorageUint<B, L> {
    type Wraps<'a> = Uint<B, L>;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = (B / 8);

    unsafe fn new(slot: U256, offset: u8) -> Self {
        debug_assert!(B <= 256);
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a, const B: usize, const L: usize> SimpleStorageType<'a> for StorageUint<B, L> {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<const B: usize, const L: usize> Erasable for StorageUint<B, L> {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl<const B: usize, const L: usize> Deref for StorageUint<B, L> {
    type Target = Uint<B, L>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { StorageCache::get_uint(self.slot, self.offset.into()) })
    }
}

impl<const B: usize, const L: usize> From<StorageUint<B, L>> for Uint<B, L> {
    fn from(value: StorageUint<B, L>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`Signed`].
#[derive(Debug)]
pub struct StorageSigned<const B: usize, const L: usize> {
    slot: U256,
    offset: u8,
    cached: OnceCell<Signed<B, L>>,
}

impl<const B: usize, const L: usize> StorageSigned<B, L> {
    /// Gets the underlying [`Signed`] in persistent storage.
    pub fn get(&self) -> Signed<B, L> {
        **self
    }

    /// Gets the underlying [`Signed`] in persistent storage.
    pub fn set(&mut self, value: Signed<B, L>) {
        overwrite_cell(&mut self.cached, value);
        unsafe { StorageCache::set_signed(self.slot, self.offset.into(), value) };
    }
}

impl<const B: usize, const L: usize> StorageType for StorageSigned<B, L> {
    type Wraps<'a> = Signed<B, L>;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = (B / 8);

    unsafe fn new(slot: U256, offset: u8) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a, const B: usize, const L: usize> SimpleStorageType<'a> for StorageSigned<B, L> {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<const B: usize, const L: usize> Erasable for StorageSigned<B, L> {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO)
    }
}

impl<const B: usize, const L: usize> Deref for StorageSigned<B, L> {
    type Target = Signed<B, L>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { StorageCache::get_signed(self.slot, self.offset.into()) })
    }
}

impl<const B: usize, const L: usize> From<StorageSigned<B, L>> for Signed<B, L> {
    fn from(value: StorageSigned<B, L>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`FixedBytes`].
#[derive(Debug)]
pub struct StorageFixedBytes<const N: usize> {
    slot: U256,
    offset: u8,
    cached: OnceCell<FixedBytes<N>>,
}

impl<const N: usize> StorageFixedBytes<N> {
    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn get(&self) -> FixedBytes<N> {
        **self
    }

    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn set(&mut self, value: FixedBytes<N>) {
        overwrite_cell(&mut self.cached, value);
        unsafe { StorageCache::set(self.slot, self.offset.into(), value) }
    }
}

impl<const N: usize> StorageType for StorageFixedBytes<N> {
    type Wraps<'a> = FixedBytes<N>;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = N;

    unsafe fn new(slot: U256, offset: u8) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a, const N: usize> SimpleStorageType<'a> for StorageFixedBytes<N> {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<const N: usize> Erasable for StorageFixedBytes<N> {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO)
    }
}

impl<const N: usize> Deref for StorageFixedBytes<N> {
    type Target = FixedBytes<N>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { StorageCache::get(self.slot, self.offset.into()) })
    }
}

impl<const N: usize> From<StorageFixedBytes<N>> for FixedBytes<N> {
    fn from(value: StorageFixedBytes<N>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`bool`].
#[derive(Debug)]
pub struct StorageBool {
    slot: U256,
    offset: u8,
    cached: OnceCell<bool>,
}

impl StorageBool {
    /// Gets the underlying [`bool`] in persistent storage.
    pub fn get(&self) -> bool {
        **self
    }

    /// Gets the underlying [`bool`] in persistent storage.
    pub fn set(&mut self, value: bool) {
        overwrite_cell(&mut self.cached, value);
        unsafe { StorageCache::set_byte(self.slot, self.offset.into(), value as u8) }
    }
}

impl StorageType for StorageBool {
    type Wraps<'a> = bool;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = 1;

    unsafe fn new(slot: U256, offset: u8) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a> SimpleStorageType<'a> for StorageBool {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl Erasable for StorageBool {
    fn erase(&mut self) {
        self.set(false);
    }
}

impl Deref for StorageBool {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            let data = StorageCache::get_byte(self.slot, self.offset.into());
            data != 0
        })
    }
}

impl From<StorageBool> for bool {
    fn from(value: StorageBool) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`Address`].
#[derive(Debug)]
pub struct StorageAddress {
    slot: U256,
    offset: u8,
    cached: OnceCell<Address>,
}

impl StorageAddress {
    /// Gets the underlying [`Address`] in persistent storage.
    pub fn get(&self) -> Address {
        **self
    }

    /// Gets the underlying [`Address`] in persistent storage.
    pub fn set(&mut self, value: Address) {
        overwrite_cell(&mut self.cached, value);
        unsafe { StorageCache::set::<20>(self.slot, self.offset.into(), value.into()) }
    }
}

impl StorageType for StorageAddress {
    type Wraps<'a> = Address;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = 20;

    unsafe fn new(slot: U256, offset: u8) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a> SimpleStorageType<'a> for StorageAddress {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl Erasable for StorageAddress {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl Deref for StorageAddress {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            StorageCache::get::<20>(self.slot, self.offset.into()).into()
        })
    }
}

impl From<StorageAddress> for Address {
    fn from(value: StorageAddress) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`BlockNumber`].
#[derive(Debug)]
pub struct StorageBlockNumber {
    slot: U256,
    offset: u8,
    cached: OnceCell<BlockNumber>,
}

impl StorageBlockNumber {
    /// Gets the underlying [`BlockNumber`] in persistent storage.
    pub fn get(&self) -> BlockNumber {
        **self
    }

    /// Gets the underlying [`BlockNumber`] in persistent storage.
    pub fn set(&mut self, value: BlockNumber) {
        overwrite_cell(&mut self.cached, value);
        let value = FixedBytes::from(value.to_be_bytes());
        unsafe { StorageCache::set::<8>(self.slot, self.offset.into(), value) };
    }
}

impl StorageType for StorageBlockNumber {
    type Wraps<'a> = BlockNumber;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = 8;

    unsafe fn new(slot: U256, offset: u8) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a> SimpleStorageType<'a> for StorageBlockNumber {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl Erasable for StorageBlockNumber {
    fn erase(&mut self) {
        self.set(0);
    }
}

impl Deref for StorageBlockNumber {
    type Target = BlockNumber;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            let data = StorageCache::get::<8>(self.slot, self.offset.into());
            u64::from_be_bytes(data.0)
        })
    }
}

impl From<StorageBlockNumber> for BlockNumber {
    fn from(value: StorageBlockNumber) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`BlockHash`].
#[derive(Clone, Debug)]
pub struct StorageBlockHash {
    slot: U256,
    cached: OnceCell<BlockHash>,
}

impl StorageBlockHash {
    /// Gets the underlying [`BlockHash`] in persistent storage.
    pub fn get(&self) -> BlockHash {
        **self
    }

    /// Sets the underlying [`BlockHash`] in persistent storage.
    pub fn set(&mut self, value: BlockHash) {
        overwrite_cell(&mut self.cached, value);
        unsafe { StorageCache::set_word(self.slot, value) }
    }
}

impl StorageType for StorageBlockHash {
    type Wraps<'a> = BlockHash;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    unsafe fn new(slot: U256, _offset: u8) -> Self {
        let cached = OnceCell::new();
        Self { slot, cached }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a> SimpleStorageType<'a> for StorageBlockHash {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl Erasable for StorageBlockHash {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl Deref for StorageBlockHash {
    type Target = BlockHash;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| StorageCache::get_word(self.slot))
    }
}

impl From<StorageBlockHash> for BlockHash {
    fn from(value: StorageBlockHash) -> Self {
        *value
    }
}