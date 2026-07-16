use arrow_schema::Field;
// use rastertile_schema::{Metadata, RasterType};

pub fn has_extension(value: &Field) -> bool {
    
    value.extension_type_metadata().is_some()
}

// pub fn get_input_array(){
    
// }
