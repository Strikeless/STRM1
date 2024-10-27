use std::ops::Range;

use super::VarKey;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RangedUsageMap {
    slots: Vec<UsageSlot>,
    usable_region: Range<usize>,
}

impl RangedUsageMap {
    pub fn new(capacity: usize) -> Self {
        Self::new_with_usable_region(0..capacity)
    }

    pub fn new_with_usable_region(usable_region: Range<usize>) -> Self {
        Self {
            slots: Vec::new(),
            usable_region,
        }
    }

    pub fn preallocated(mut self) -> Self {
        self.slots = vec![UsageSlot::default(); self.capacity()];
        self
    }

    pub fn reserve(&mut self, slot: usize, range: Range<usize>, var: VarKey) {
        let slot = self.slot_mut(slot);
        slot.reserve(range, var);
    }

    pub fn free(&mut self, slot: usize, range: &Range<usize>, var: &VarKey) {
        let slot = self.slot_mut(slot);
        slot.free(range, var);
    }

    pub fn free_slot(&mut self, range: &Range<usize>) -> Option<usize> {
        let existing_free_index = self
            .slots
            .iter()
            .position(|usage| usage.is_free_for_range(range));

        existing_free_index
            .or_else(|| self.create_new_slot())
            .map(|index| self.vec_index_to_slot(index))
    }

    pub fn capacity(&self) -> usize {
        self.usable_region.len()
    }

    fn create_new_slot(&mut self) -> Option<usize> {
        if self.is_full() {
            return None;
        }

        self.slots.push(UsageSlot::default());
        Some(self.slots.len() - 1)
    }

    fn is_full(&self) -> bool {
        self.slots.len() == self.capacity()
    }

    fn slot_mut(&mut self, slot: usize) -> &mut UsageSlot {
        let index = self.slot_to_vec_index(slot);
        self.slots.get_mut(index).unwrap()
    }

    fn slot_to_vec_index(&self, slot: usize) -> usize {
        slot - self.usable_region.start
    }

    fn vec_index_to_slot(&self, index: usize) -> usize {
        index + self.usable_region.start
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct UsageSlot {
    ranged_usages: Vec<(Range<usize>, VarKey)>,
}

impl UsageSlot {
    pub fn reserve(&mut self, range: Range<usize>, var: VarKey) {
        let reservation = (range, var);

        if self.ranged_usages.contains(&reservation) {
            panic!("Complete duplicate range-var reservation");
        }

        self.ranged_usages.push(reservation);
    }

    pub fn free(&mut self, range: &Range<usize>, var: &VarKey) {
        let index = self
            .ranged_usages
            .iter()
            .position(|(found_range, found_var)| (found_range, found_var) == (range, var))
            .expect("Tried to free variable on non-reserved range");

        self.ranged_usages.remove(index);
    }

    pub fn is_free_for_range(&self, range: &Range<usize>) -> bool {
        self.ranged_usages
            .iter()
            .all(|(used_range, _)| !Self::ranges_overlap(range, used_range))
    }

    fn ranges_overlap(a: &Range<usize>, b: &Range<usize>) -> bool {
        a.start <= b.end && b.start <= a.end
    }
}
