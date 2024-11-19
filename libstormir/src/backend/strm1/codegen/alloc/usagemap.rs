use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RangedUsageMap<V>
where
    V: Clone + PartialEq,
{
    slots: Vec<UsageSlot<V>>,
    usable_region: Range<usize>,
}

impl<V> RangedUsageMap<V>
where
    V: Clone + PartialEq,
{
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

    pub fn reserve(&mut self, slot: usize, range: Range<usize>, value: V) {
        let slot = self.slot_mut(slot);
        slot.reserve(range, value);
    }

    pub fn free(&mut self, slot: usize, range: &Range<usize>, value: &V) {
        let slot = self.slot_mut(slot);
        slot.free(range, value);
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

    fn slot_mut(&mut self, slot: usize) -> &mut UsageSlot<V> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct UsageSlot<V>
where
    V: Clone + PartialEq,
{
    ranged_usages: Vec<(Range<usize>, V)>,
}

impl<V> Default for UsageSlot<V>
where
    V: Clone + PartialEq,
{
    fn default() -> Self {
        Self {
            ranged_usages: Vec::new(),
        }
    }
}

impl<V> UsageSlot<V>
where
    V: Clone + PartialEq,
{
    pub fn reserve(&mut self, range: Range<usize>, value: V) {
        let reservation = (range, value);

        if self.ranged_usages.contains(&reservation) {
            panic!("Complete duplicate range-var reservation");
        }

        self.ranged_usages.push(reservation);
    }

    pub fn free(&mut self, range: &Range<usize>, value: &V) {
        let index = self
            .ranged_usages
            .iter()
            .position(|(found_range, found_value)| (found_range, found_value) == (range, value))
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
