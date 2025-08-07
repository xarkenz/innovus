#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ArenaHandle {
    pub slot: usize,
    pub version: usize,
}

#[derive(Debug)]
pub struct UnboundedArena<T> {
    slots: Vec<ArenaSlot<T>>,
    next_open_slot: Option<usize>,
}

impl<T> UnboundedArena<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            next_open_slot: None,
        }
    }

    pub fn slots(&self) -> &[ArenaSlot<T>] {
        &self.slots
    }

    pub fn insert(&mut self, value: T) -> ArenaHandle {
        if let Some(slot) = self.next_open_slot {
            self.slots[slot].convert_to_filled(&mut self.next_open_slot, slot, value)
        }
        else {
            let handle = ArenaHandle {
                slot: self.slots.len(),
                version: 0,
            };
            self.slots.push(ArenaSlot::Filled {
                value,
                version: 0,
            });
            handle
        }
    }

    pub fn get(&self, handle: ArenaHandle) -> Option<&T> {
        self.slots.get(handle.slot)?.value(handle.version)
    }

    pub fn get_mut(&mut self, handle: ArenaHandle) -> Option<&mut T> {
        self.slots.get_mut(handle.slot)?.value_mut(handle.version)
    }

    pub fn get_current(&self, slot: usize) -> Option<&T> {
        self.slots.get(slot)?.current_value()
    }

    pub fn get_current_mut(&mut self, slot: usize) -> Option<&mut T> {
        self.slots.get_mut(slot)?.current_value_mut()
    }

    pub fn remove(&mut self, handle: ArenaHandle) -> Option<T> {
        self.slots.get_mut(handle.slot)?.convert_to_open(&mut self.next_open_slot, handle)
    }

    pub fn values(&self) -> ArenaValues<'_, T> {
        ArenaValues::new(&self.slots)
    }

    pub fn values_mut(&mut self) -> ArenaValuesMut<'_, T> {
        ArenaValuesMut::new(&mut self.slots)
    }
}

#[derive(Debug)]
pub struct BoundedArena<T> {
    slots: Box<[ArenaSlot<T>]>,
    next_open_slot: Option<usize>,
}

impl<T> BoundedArena<T> {
    pub fn new(max_value_count: usize) -> Self {
        if max_value_count == 0 {
            panic!("cannot create a bounded arena with a max_value_count of 0");
        }
        Self {
            slots: (1..max_value_count)
                .map(Some)
                .chain(std::iter::once(None))
                .map(|next_open_slot| ArenaSlot::Open {
                    next_open_slot,
                    version: 0,
                })
                .collect(),
            next_open_slot: Some(0),
        }
    }

    pub fn slots(&self) -> &[ArenaSlot<T>] {
        &self.slots
    }

    pub fn try_insert(&mut self, value: T) -> Result<ArenaHandle, T> {
        if let Some(slot) = self.next_open_slot {
            Ok(self.slots[slot].convert_to_filled(&mut self.next_open_slot, slot, value))
        }
        else {
            Err(value)
        }
    }

    pub fn get(&self, handle: ArenaHandle) -> Option<&T> {
        self.slots.get(handle.slot)?.value(handle.version)
    }

    pub fn get_mut(&mut self, handle: ArenaHandle) -> Option<&mut T> {
        self.slots.get_mut(handle.slot)?.value_mut(handle.version)
    }

    pub fn get_current(&self, slot: usize) -> Option<&T> {
        self.slots.get(slot)?.current_value()
    }

    pub fn get_current_mut(&mut self, slot: usize) -> Option<&mut T> {
        self.slots.get_mut(slot)?.current_value_mut()
    }

    pub fn remove(&mut self, handle: ArenaHandle) -> Option<T> {
        self.slots.get_mut(handle.slot)?.convert_to_open(&mut self.next_open_slot, handle)
    }

    pub fn values(&self) -> ArenaValues<'_, T> {
        ArenaValues::new(&self.slots)
    }

    pub fn values_mut(&mut self) -> ArenaValuesMut<'_, T> {
        ArenaValuesMut::new(&mut self.slots)
    }
}

#[derive(Debug)]
pub enum ArenaSlot<T> {
    Open {
        next_open_slot: Option<usize>,
        version: usize,
    },
    Filled {
        value: T,
        version: usize,
    },
}

impl<T> ArenaSlot<T> {
    pub fn value(&self, handle_version: usize) -> Option<&T> {
        match self {
            Self::Open { .. } => None,
            Self::Filled { value, version } => {
                (handle_version == *version).then_some(value)
            }
        }
    }

    pub fn value_mut(&mut self, handle_version: usize) -> Option<&mut T> {
        match self {
            Self::Open { .. } => None,
            Self::Filled { value, version } => {
                (handle_version == *version).then_some(value)
            }
        }
    }

    pub fn current_value(&self) -> Option<&T> {
        match self {
            Self::Open { .. } => None,
            Self::Filled { value, .. } => Some(value),
        }
    }

    pub fn current_value_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Open { .. } => None,
            Self::Filled { value, .. } => Some(value),
        }
    }

    fn convert_to_filled(&mut self, open_slot: &mut Option<usize>, slot: usize, value: T) -> ArenaHandle {
        if let Self::Open { next_open_slot, version } = *self {
            *open_slot = next_open_slot;
            *self = Self::Filled {
                value,
                version,
            };
            ArenaHandle {
                slot,
                version,
            }
        }
        else {
            panic!("arena slot is filled despite being marked as open")
        }
    }

    fn convert_to_open(&mut self, open_slot: &mut Option<usize>, handle: ArenaHandle) -> Option<T> {
        if let Self::Filled { version, .. } = *self {
            if handle.version != version {
                None
            }
            else if let Self::Filled { value, .. } = std::mem::replace(
                self,
                Self::Open {
                    next_open_slot: *open_slot,
                    version: version.wrapping_add(1),
                },
            ) {
                *open_slot = Some(handle.slot);
                Some(value)
            }
            else {
                // We literally just checked that the slot was filled
                unreachable!()
            }
        }
        else {
            None
        }
    }
}

pub struct ArenaValues<'a, T> {
    slots: std::slice::Iter<'a, ArenaSlot<T>>,
    next_slot: usize,
}

impl<'a, T> ArenaValues<'a, T> {
    fn new(slots: &'a [ArenaSlot<T>]) -> Self {
        Self {
            slots: slots.iter(),
            next_slot: 0,
        }
    }
}

impl<'a, T> Iterator for ArenaValues<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let slot = self.next_slot;
            self.next_slot += 1;
            match self.slots.next() {
                Some(ArenaSlot::Open { .. }) => continue,
                Some(ArenaSlot::Filled { value, .. }) => break Some((slot, value)),
                None => break None,
            }
        }
    }
}

pub struct ArenaValuesMut<'a, T> {
    slots: std::slice::IterMut<'a, ArenaSlot<T>>,
    next_slot: usize,
}

impl<'a, T> ArenaValuesMut<'a, T> {
    fn new(slots: &'a mut [ArenaSlot<T>]) -> Self {
        Self {
            slots: slots.iter_mut(),
            next_slot: 0,
        }
    }
}

impl<'a, T> Iterator for ArenaValuesMut<'a, T> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let slot = self.next_slot;
            self.next_slot += 1;
            match self.slots.next() {
                Some(ArenaSlot::Open { .. }) => continue,
                Some(ArenaSlot::Filled { value, .. }) => break Some((slot, value)),
                None => break None,
            }
        }
    }
}
