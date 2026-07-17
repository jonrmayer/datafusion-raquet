// pub mod converter;

mod quadbin_from_lonlat;
mod quadbin_from_tile;
mod quadbin_kring;

mod quadbin_resolution;
mod quadbin_sibling;
mod quadbin_to_bbox;
mod quadbin_to_bbox_mercator;
mod quadbin_to_bbox_wgs84;
mod quadbin_to_children;
mod quadbin_to_geojson;
mod quadbin_to_lonlat;
mod quadbin_to_parent;
mod quadbin_to_tile;
mod quadbin_to_wkt;

#[cfg(any(test, debug_assertions))]
pub mod testing;

pub use quadbin_from_lonlat::QuadBinFromLonLat;
pub use quadbin_from_tile::QuadBinFromTile;
pub use quadbin_kring::QuadBinKRing;

pub use quadbin_resolution::QuadBinResolution;
pub use quadbin_sibling::QuadBinToSibling;
pub use quadbin_to_bbox::QuadBinToBBOX;
pub use quadbin_to_bbox_mercator::QuadBinToBBOXMercator;
pub use quadbin_to_bbox_wgs84::QuadBinToBBOXWGS84;
pub use quadbin_to_children::QuadBinToChildren;
pub use quadbin_to_geojson::QuadBinToGeoJSON;
pub use quadbin_to_lonlat::QuadBinToLonLat;
pub use quadbin_to_parent::QuadBinToParent;
pub use quadbin_to_tile::QuadBinToTile;
pub use quadbin_to_wkt::QuadBinToWKT;

pub fn register(session_context: &datafusion::prelude::SessionContext) {
    session_context.register_udf(QuadBinFromTile::default().into());
    session_context.register_udf(QuadBinToTile::default().into());
    session_context.register_udf(QuadBinFromLonLat::default().into());
    session_context.register_udf(QuadBinToLonLat::default().into());
    session_context.register_udf(QuadBinToParent::default().into());
    session_context.register_udf(QuadBinResolution::default().into());
    session_context.register_udf(QuadBinToChildren::default().into());
    session_context.register_udf(QuadBinToSibling::default().into());
    session_context.register_udf(QuadBinKRing::default().into());

    session_context.register_udf(QuadBinToBBOX::default().into());
    session_context.register_udf(QuadBinToBBOXWGS84::default().into());
    session_context.register_udf(QuadBinToBBOXMercator::default().into());
    session_context.register_udf(QuadBinToWKT::default().into());
    session_context.register_udf(QuadBinToGeoJSON::default().into());
}
