

type EntityID = u32;

pub struct Entity {
    id: EntityID,
}

impl Entity {

    pub fn id(&self) -> EntityID {
        self.id
    }

}


pub struct Scene {
    entities: Vec<EntityID>,
}

impl Scene {

    pub fn create_entity(&mut self) -> Entity {
        const FIRST_ENTITY_ID: EntityID = 1;
        let mut new_id: EntityID = FIRST_ENTITY_ID;
        let mut insert_index = self.entities.len();
        match self.entities.last() {
            None => {},
            Some(last_id) => {
                new_id = *last_id;
                for (index, id) in self.entities.iter().enumerate() {
                    if *id != FIRST_ENTITY_ID + index as EntityID {
                        new_id = FIRST_ENTITY_ID + index as EntityID;
                        insert_index = index;
                        break;
                    }
                }
            }
        }
        self.entities.insert(insert_index, new_id);
        Entity{ id: new_id }
    }

    pub fn get_entity(id: EntityID) -> Entity {
        Entity{ id }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        self.remove_entity_id(entity.id());
    }

    pub fn remove_entity_id(&mut self, entity_id: EntityID) {
        for (index, id) in self.entities.iter().enumerate() {
            if *id == entity_id {
                self.entities.remove(index);
                return;
            }
        }
    }

}