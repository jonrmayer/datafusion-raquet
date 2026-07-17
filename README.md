# Apache Datafusion Raquet Extension

An Apache Datafusion extension for working with [Raquet](https://www.raquet.io/) raster data stored in Apache Parquet format with QUADBIN spatial indexing.

## What is Raquet?

**Raquet** is a specification created by [CARTO](https://carto.com) for storing raster data efficiently using:

- **Apache Parquet** - Columnar storage for efficient compression and query performance
- **QUADBIN** - A spatial indexing scheme encoding Web Mercator tile coordinates into 64-bit integers
- **Binary Band Data** - Band values stored as compressed BLOBs (gzip, JPEG, or WebP)

This extension enables Datafusion to query Raquet files directly using SQL, with functions for spatial indexing and pixel extraction.


### QUADBIN Functions

| Function | Description | Return |
|----------|-------------|--------|
| `quadbin_from_tile(x, y, z)` | Tile coordinates to QUADBIN | `UBIGINT` |
| `quadbin_to_tile(cell)` | QUADBIN to tile coordinates | `STRUCT(x, y, z)` |
| `quadbin_from_lonlat(lon, lat, resolution)` | Lon/lat to QUADBIN cell | `UBIGINT` |
| `quadbin_to_lonlat(cell)` | Cell center as lon/lat | `STRUCT(lon, lat)` |
| `quadbin_resolution(cell)` | Get resolution level | `INTEGER` |
| `quadbin_to_bbox(cell)` | Bounding box of cell | `STRUCT(...)` |
| `quadbin_to_parent(cell)` | Parent cell (resolution - 1) | `UBIGINT` |
| `quadbin_to_parent(cell, resolution)` | Parent at specific resolution | `UBIGINT` |
| `quadbin_to_children(cell)` | 4 children at resolution + 1 | `LIST(UBIGINT)` |
| `quadbin_to_children(cell, resolution)` | Children at specific resolution | `LIST(UBIGINT)` |
| `quadbin_sibling(cell)` | Sibling cells (same parent) | `LIST(UBIGINT)` |
| `quadbin_kring(cell, k)` | Cells within k distance | `LIST(UBIGINT)` |
| `quadbin_to_wkt(cell)` | Cell as WKT POLYGON | `VARCHAR` |
| `quadbin_to_geojson(cell)` | Cell as GeoJSON | `VARCHAR` |


### Raster Functions

| Function | Description | Return |
|----------|-------------|--------|
| `raquet_decompress(band)` | Decompress band bytes | `[]byte` |
| `raquet_band_decode(band)` | Decompress + Decode band bytes to []Float64 | `[]Float64` |
| `raquet_band_native(band)` | Decompress + Decode band bytes to native data type eg []Int16 | `[]Native` |
| `raquet_band_statistics(band)` | Return a summary statistics struct from a band | `STRUCT(valid_count, sum, mean, min, max, stddev)` |
| `raquet_pixel(band,pixel_x,pixel_y)` | Pixel Value by Tile x,y | `Float64` |
| `raquet_value(band,wkt,resolution)` | Pixel Value by Point WKT and resolution | `Float64` |


### Utility Functions


| Function | Description | Return |
|----------|-------------|--------|
| `band_metadata(band_name,metadata)` | Return a metadata struct for a named band | `STRUCT(tile_size, binary_type, data_type, no_data, compression)` |
| `quadbin_metadata(metadata)` | Return the metadata struct for the quadbin column | `STRUCT(min_zoom, max_zoom)` |
| `binary_to_raquet(band,tile_size, binary_type, data_type,no_data,compression)` | Return a raquet band from a binary | `[]byte` |
| `quadbin_pixel_xy(lon, lat, res, tile_size)` | Pixel coordinates within tile | `STRUCT(pixel_x, pixel_y)` |
| `quadbin_polyfill(wkt, resolution)` | Fill geometry from wkt with cells | `LIST(UBIGINT)` |




