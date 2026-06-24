
import os
import duckdb
from datafusion import SessionConfig
from pyarrow import Table
from datafusion_raquet import RaquetSessionContext
from dotenv import load_dotenv
from pathlib import Path
import time

dotenv_path = Path('/home/jonrm/projects/git/datafusion-raquet/.env')
RAQUET_DATA_HOME_DIR = os.getenv('RAQUET_DATA_HOME_DIR')





def timer(func):
    def wrapper(*args, **kwargs):
        nonlocal total
        start = time.time()
        result = func(*args, **kwargs)
        duration = time.time() - start
        total += duration
        print(
            f"Execution time: {func.__name__} Execution time: {duration}   Total: {total}"
        )
        return result

    total = 0
    return wrapper


config = SessionConfig().with_information_schema(True)

ctx = RaquetSessionContext(config=config)
ctx.register_all_quadbin()
ctx.register_raquet(
    "solar",
    "{}spain_solar_ghi.parquet".format(RAQUET_DATA_HOME_DIR),
)

sql = """
INSTALL raquet FROM community;
LOAD raquet;
"""
duckdb.execute(sql)


@timer
def duckdb_quadbin_to_tile():
    """ """

    sql = """
   
    select quadbin_to_tile(5202642732031410175) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_to_tile():
    """ """
    sql = """    
    select quadbin_to_tile(5202642732031410175) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_to_wkt():
    """ """

    sql = """
   
    select quadbin_to_wkt(5202642732031410175) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_to_wkt():
    """ """
    sql = """    
    select quadbin_to_wkt(5202642732031410175) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_to_geojson():
    """ """

    sql = """
   
    select quadbin_to_geojson(5202642732031410175) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_to_geojson():
    """ """
    sql = """    
    select quadbin_to_geojson(5202642732031410175) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_resolution():
    """ """

    sql = """
   
    select quadbin_resolution(5202642732031410175) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_resolution():
    """ """
    sql = """    
    select quadbin_resolution(5202642732031410175) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_sibling():
    """ """

    sql = """
   
    select quadbin_sibling(5202642732031410175) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_sibling():
    """ """
    sql = """    
    select quadbin_sibling(5202642732031410175) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_to_parent():
    """ """

    sql = """
   
    select quadbin_to_parent(5202642732031410175) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_to_parent():
    """ """
    sql = """    
    select quadbin_to_parent(5202642732031410175) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_to_parent_variant():
    """ """

    sql = """
   
    select quadbin_to_parent(5202642732031410175,1) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_to_parent_variant():
    """ """
    sql = """    
    select quadbin_to_parent(5202642732031410175,1) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_to_children():
    """ """

    sql = """
   
    select quadbin_to_children(5202642732031410175) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_to_children():
    """ """
    sql = """    
    select quadbin_to_children(5202642732031410175) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_to_children_variant():
    """ """

    sql = """
   
    select quadbin_to_children(5202642732031410175,5) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_to_children_variant():
    """ """
    sql = """    
    select quadbin_to_children(5202642732031410175,5) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_kring():
    """ """

    sql = """
   
    select quadbin_kring(5202642732031410175,1) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_kring():
    """ """
    sql = """    
    select quadbin_kring(5202642732031410175,1) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_from_tile():
    """ """

    sql = """
   
    select quadbin_from_tile(0, 0, 0) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_from_tile():
    """ """
    sql = """    
    select quadbin_from_tile(0, 0, 0) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_from_lonlat():
    """ """

    sql = """
   
    select quadbin_from_lonlat(0, 0, 0) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_from_lonlat():
    """ """
    sql = """    
    select quadbin_from_lonlat(0, 0, 0) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_to_lonlat():
    """ """

    sql = """
   
    select quadbin_to_lonlat(5202642732031410175) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_to_lonlat():
    """ """
    sql = """    
    select quadbin_to_lonlat(5202642732031410175) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_to_bbox():
    """ """

    sql = """
   
    select quadbin_to_bbox(5202642732031410175) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_to_bbox():
    """ """
    sql = """    
    select quadbin_to_bbox(5202642732031410175) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_polyfill():
    """ """

    sql = """
   
    with data as(
    select unnest(quadbin_polyfill('POLYGON((-74.1 40.6, -73.8 40.6, -73.8 40.9, -74.1 40.9, -74.1 40.6))'::GEOMETRY,11)) as cells
    )
    select cells,quadbin_to_wkt(cells) from data 
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_polyfill():
    """ """
    sql = """ 
    with data as(
    select unnest(quadbin_polyfill('POLYGON((-74.1 40.6, -73.8 40.6, -73.8 40.9, -74.1 40.9, -74.1 40.6))',11)) cells
    
    )

    select cells,quadbin_to_wkt(cast(cells as bigint)) from data 
    """
    decoded = ctx.sql(sql)

    decoded.show()


@timer
def duckdb_quadbin_pixel_xy():
    """ """

    sql = """
   
    select quadbin_pixel_xy(0.0, 0.0, 4, 256) as output
    """
    duckdb.sql(sql).show()


@timer
def datafusion_quadbin_pixel_xy():
    """ """
    sql = """    
    select quadbin_pixel_xy(0.0, 0.0, 4, 256) as output
    """
    decoded = ctx.sql(sql)

    decoded.show()


# duckdb_quadbin_resolution()
# datafusion_quadbin_resolution()

# duckdb_quadbin_sibling()
# datafusion_quadbin_sibling()

# duckdb_quadbin_to_parent()
# datafusion_quadbin_to_parent()

# duckdb_quadbin_to_parent_variant()
# datafusion_quadbin_to_parent_variant()

# duckdb_quadbin_to_children()
# datafusion_quadbin_to_children()

# duckdb_quadbin_to_children_variant()
# datafusion_quadbin_to_children_variant()

# duckdb_quadbin_kring()
# datafusion_quadbin_kring()

# duckdb_quadbin_from_tile()
# datafusion_quadbin_from_tile()


# duckdb_quadbin_to_tile()
# datafusion_quadbin_to_tile()

# duckdb_quadbin_from_lonlat()
# datafusion_quadbin_from_lonlat()


# duckdb_quadbin_to_lonlat()
# datafusion_quadbin_to_lonlat()


# duckdb_quadbin_to_wkt()
# datafusion_quadbin_to_wkt()


# duckdb_quadbin_to_geojson()
# datafusion_quadbin_to_geojson()


# duckdb_quadbin_to_bbox()
# datafusion_quadbin_to_bbox()


# # duckdb_quadbin_polyfill()
# # datafusion_quadbin_polyfill()


# duckdb_quadbin_pixel_xy()
# datafusion_quadbin_pixel_xy()


@timer
def performance_duckdb_quadbin_pixel_xy():
    sql = """ 
    with test as 
    (
    select unnest(generate_series(1,100000,1)) as output
    ),
    result as 
    (
    select test.output,quadbin_pixel_xy(0.0, 0.0, 4, 256) pixel_xy from test    
    )
    select count(*)  from result

        """

    duckdb.sql(sql).show()


@timer
def performance_datafusion_quadbin_pixel_xy():
    sql = """ 

    with test as 
    (
    select unnest(generate_series(1,100000)) as output
    ),
    result as 
    (
    select test.output,select quadbin_pixel_xy(0.0, 0.0, 4, 256) pixel_xy from test    
    )
    select count(*)  from result
    """

    ctx.sql(sql).show()


@timer
def performance_duckdb_quadbin_to_bbox():
    sql = """ 
    with test as 
    (
    select unnest(generate_series(1,1000,1)) as output
    ),
    result as 
    (
    select test.output,quadbin_to_bbox(5202642732031410175) as bbox from test  
    )
    select count(*)  from result

        """

    duckdb.sql(sql).show()


@timer
def performance_datafusion_quadbin_to_bbox():
    sql = """ 

    with test as 
    (
    select unnest(generate_series(1,1000)) as output
    ),
    test_input as (
    select output,0.0 a, 0.0 b, 4 c, 256 d from test
    ),
    result as 
    (
    select output,quadbin_pixel_xy(a, b, c, d) pixel_xy from test_input    
    )
    select count(*)  from result
    """

    ctx.sql(sql).show()


# performance_duckdb_quadbin_pixel_xy()
# performance_datafusion_quadbin_pixel_xy()

# performance_duckdb_quadbin_to_bbox()
# performance_datafusion_quadbin_to_bbox()



