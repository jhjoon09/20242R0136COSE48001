use uuid::Uuid;

pub enum MetaEvent {
    Propagation { group: Uuid },
    Register { group: Uuid, id: Uuid },
}
