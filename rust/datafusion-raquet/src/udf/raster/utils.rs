use arrow_schema::Field;
// use rastertile_schema::{Metadata, RasterType};

pub fn has_extension(value: &Field) -> bool {
    let extension = match value.extension_type_metadata() {
        Some(_) => true,
        None => false,
    };
    extension
}

// pub fn get_input_array(){
    
// }
