use crate::id_type;

id_type!(UserId);
id_type!(AuditId);
id_type!(VisualId);

mod admin_id;
pub use admin_id::AdminId;
