import duckdb
from datafusion import SessionConfig
from pyarrow import Table
from datafusion_raquet import RaquetSessionContext

import time

def timer(func):
    def wrapper(*args, **kwargs):
        nonlocal total
        start = time.time()
        result = func(*args, **kwargs)
        duration = time.time() - start
        total += duration
        print(f"Execution time: {duration}   Total: {total}")
        return result

    total = 0
    return wrapper

config = SessionConfig().with_information_schema(True)

ctx = RaquetSessionContext(config=config)
ctx.register_all_quadbin()
ctx.register_rastertile()
ctx.register_raquet(
    "solar",
    "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet",
)

ctx.register_raquet(
    "tci",
    "/home/jonrm/projects/git/raquet-datafusion/data/parquet/tci_interleaved_gzip.parquet",
)

# sql = """
# INSTALL raquet FROM community;
# """
# duckdb.execute(sql)

sql = """
with data as (
select array_length(native_tile(pixels),1) pl from tci where block<>0
)

select sum(pl) from data


"""


sql = """
LOAD raquet;
"""
# duckdb.sql(sql)


@timer
def raquet():
    sql = """
    with data as (
    SELECT raquet_pixel(band_1,124,124) as pixel_value from solar where block<>0 
    --SELECT native_tile(band_1) as pixel_value from solar where block<>0 
    )

    select count(*) from data


    """
    decoded = ctx.sql(sql)
    decoded.show()

@timer
def db():

    sql = """
    LOAD raquet;
    with data as (
    --select raquet_decode_band(band_1, 'float32', 256, 256, 'gzip') as band_data
    select raquet_pixel(band_1,metadata,124,124) as pixel_value 
    FROM read_raquet('/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet') 
    )
    select count(*) from data

    """
    duckdb.sql(sql).show()



db()
raquet()

# def duckdb_quadbin_to_tile():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_to_tile(5202642732031410175) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_to_tile():
#     """ """
#     sql = """    
#     select quadbin_to_tile(5202642732031410175) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_to_wkt():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_to_wkt(5202642732031410175) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_to_wkt():
#     """ """
#     sql = """    
#     select quadbin_to_wkt(5202642732031410175) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_to_geojson():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_to_geojson(5202642732031410175) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_to_geojson():
#     """ """
#     sql = """    
#     select quadbin_to_geojson(5202642732031410175) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_resolution():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_resolution(5202642732031410175) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_resolution():
#     """ """
#     sql = """    
#     select quadbin_resolution(5202642732031410175) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_sibling():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_sibling(5202642732031410175) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_sibling():
#     """ """
#     sql = """    
#     select quadbin_sibling(5202642732031410175) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_to_parent():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_to_parent(5202642732031410175) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_to_parent():
#     """ """
#     sql = """    
#     select quadbin_to_parent(5202642732031410175) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_to_parent_variant():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_to_parent(5202642732031410175,1) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_to_parent_variant():
#     """ """
#     sql = """    
#     select quadbin_to_parent(5202642732031410175,1) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_to_children():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_to_children(5202642732031410175) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_to_children():
#     """ """
#     sql = """    
#     select quadbin_to_children(5202642732031410175) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_to_children_variant():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_to_children(5202642732031410175,5) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_to_children_variant():
#     """ """
#     sql = """    
#     select quadbin_to_children(5202642732031410175,5) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_kring():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_kring(5202642732031410175,1) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_kring():
#     """ """
#     sql = """    
#     select quadbin_kring(5202642732031410175,1) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_from_tile():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_from_tile(0, 0, 0) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_from_tile():
#     """ """
#     sql = """    
#     select quadbin_from_tile(0, 0, 0) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_from_lonlat():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_from_lonlat(0, 0, 0) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_from_lonlat():
#     """ """
#     sql = """    
#     select quadbin_from_lonlat(0, 0, 0) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_to_lonlat():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_to_lonlat(5202642732031410175) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_to_lonlat():
#     """ """
#     sql = """    
#     select quadbin_to_lonlat(5202642732031410175) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# def duckdb_quadbin_to_bbox():
#     """ """

#     sql = """
#     LOAD raquet;
#     select quadbin_to_bbox(5202642732031410175) as output
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_to_bbox():
#     """ """
#     sql = """    
#     select quadbin_to_bbox(5202642732031410175) as output
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()

# def duckdb_quadbin_polyfill():
#     """ """

#     sql = """
#     LOAD raquet;
#     with data as(
#     select unnest(quadbin_polyfill('POLYGON((-74.1 40.6, -73.8 40.6, -73.8 40.9, -74.1 40.9, -74.1 40.6))'::GEOMETRY,11)) as cells
#     )
#     select cells,quadbin_to_wkt(cells) from data 
#     """
#     duckdb.sql(sql).show()


# def raquet_quadbin_polyfill():
#     """ """
#     sql = """ 
#     with data as(
#     select unnest(quadbin_polyfill('POLYGON((-74.1 40.6, -73.8 40.6, -73.8 40.9, -74.1 40.9, -74.1 40.6))',11)) cells
    
#     )

#     select cells,quadbin_to_wkt(cast(cells as bigint)) from data 
#     """
#     decoded = ctx.sql(sql)

#     decoded.show()


# # duckdb_quadbin_resolution()
# # raquet_quadbin_resolution()

# # duckdb_quadbin_sibling()
# # raquet_quadbin_sibling()

# # duckdb_quadbin_to_parent()
# # raquet_quadbin_to_parent()

# # duckdb_quadbin_to_parent_variant()
# # raquet_quadbin_to_parent_variant()

# # duckdb_quadbin_to_children()
# # raquet_quadbin_to_children()

# # duckdb_quadbin_to_children_variant()
# # raquet_quadbin_to_children_variant()

# # duckdb_quadbin_kring()
# # raquet_quadbin_kring()

# # duckdb_quadbin_from_tile()
# # raquet_quadbin_from_tile()


# # duckdb_quadbin_to_tile()
# # raquet_quadbin_to_tile()

# # duckdb_quadbin_from_lonlat()
# # raquet_quadbin_from_lonlat()


# # duckdb_quadbin_to_lonlat()
# # raquet_quadbin_to_lonlat()


# # duckdb_quadbin_to_wkt()
# # raquet_quadbin_to_wkt()


# # duckdb_quadbin_to_geojson()
# # raquet_quadbin_to_geojson()


# # duckdb_quadbin_to_bbox()
# # raquet_quadbin_to_bbox()


# # duckdb_quadbin_polyfill()
# # raquet_quadbin_polyfill()
