use innovus::gfx::Mesh;
use innovus::tools::{Rectangle, Vector};
use crate::gui::render::{GuiImage, GuiVertex};
use crate::gui::render::text::{TextBackground, TextLine};
use crate::tools::asset::AssetPool;
use crate::world::item::Item;

pub fn format_item_count(count: u32) -> String {
    if count <= 1 {
        String::new()
    }
    else {
        count.to_string()
    }
}

pub struct ItemSlot {
    item: Item,
    image: Option<GuiImage>,
    count_text: TextLine,
}

impl ItemSlot {
    pub fn new() -> Self {
        Self {
            item: Item::default(),
            image: None,
            count_text: TextLine::new(
                Vector([1.0, 1.0]),
                Vector::one(),
                TextBackground::DropShadow {
                    color: Vector([0.0, 0.0, 0.0, 0.8]),
                    offset: Vector([0.0, 1.0]),
                },
                String::new(),
            ),
        }
    }

    pub fn item(&self) -> &Item {
        &self.item
    }

    pub fn set_item(&mut self, item: Item) {
        self.invalidate();
        self.count_text.set_text(format_item_count(item.count()));
        self.item = item;
    }

    pub fn take_item(&mut self) -> Item {
        self.invalidate();
        self.count_text.clear_text();
        std::mem::take(&mut self.item)
    }

    pub fn invalidate(&mut self) {
        self.image = None;
        self.count_text.invalidate();
    }

    pub fn append_to_mesh(
        &mut self,
        item_layer: &mut Mesh<GuiVertex>,
        foreground_layer: &mut Mesh<GuiVertex>,
        offset: Vector<f32, 2>,
        assets: &mut AssetPool,
    ) {
        if self.image.is_none() {
            self.image = assets.get_item_image(self.item.item_type()).map(|atlas_region| {
                GuiImage::new(
                    Rectangle::from_span(Vector::zero(), Vector([16.0, 16.0])),
                    Vector::one(),
                    atlas_region,
                )
            })
        }
        if let Some(image) = &self.image {
            image.append_to_mesh(item_layer, offset)
        }
        self.count_text.append_to_mesh(foreground_layer, offset + Vector([17.0, 19.0]), assets);
    }
}

impl Default for ItemSlot {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ItemGrid {
    slots: Vec<ItemSlot>,
    column_count: usize,
    gap: Vector<f32, 2>,
}

impl ItemGrid {
    pub fn new(slot_count: usize, column_count: usize, gap: Vector<f32, 2>) -> Self {
        Self {
            slots: std::iter::from_fn(|| Some(ItemSlot::default()))
                .take(slot_count)
                .collect(),
            column_count,
            gap,
        }
    }

    pub fn slots(&self) -> &[ItemSlot] {
        &self.slots
    }

    pub fn slots_mut(&mut self) -> &mut [ItemSlot] {
        &mut self.slots
    }

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub fn slot(&self, index: usize) -> &ItemSlot {
        &self.slots[index]
    }

    pub fn slot_mut(&mut self, index: usize) -> &mut ItemSlot {
        &mut self.slots[index]
    }

    pub fn column_count(&self) -> usize {
        self.column_count
    }

    pub fn row_count(&self) -> usize {
        self.slots.len().div_ceil(self.column_count)
    }

    pub fn gap(&self) -> &Vector<f32, 2> {
        &self.gap
    }

    pub fn invalidate(&mut self) {
        for slot in &mut self.slots {
            slot.invalidate();
        }
    }

    pub fn append_to_mesh(
        &mut self,
        item_layer: &mut Mesh<GuiVertex>,
        foreground_layer: &mut Mesh<GuiVertex>,
        offset: Vector<f32, 2>,
        assets: &mut AssetPool,
    ) {
        let mut offset_y = offset.y();
        for row_slots in self.slots.chunks_mut(self.column_count) {
            let mut offset_x = offset.x();
            for slot in row_slots {
                slot.append_to_mesh(
                    item_layer,
                    foreground_layer,
                    Vector([offset_x, offset_y]),
                    assets,
                );
                offset_x += 16.0 + self.gap.x();
            }
            offset_y += 16.0 + self.gap.y();
        }
    }
}
