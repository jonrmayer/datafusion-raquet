import duckdb
from datafusion import SessionConfig
from pyarrow import Table
from datafusion_raquet import RaquetSessionContext
from dotenv import load_dotenv
from pathlib import Path
import os
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
        print(f"Execution time: {func.__name__} Execution time: {duration}   Total: {total}")
        return result

    total = 0
    return wrapper

config = SessionConfig().with_information_schema(True)

ctx = RaquetSessionContext(config=config)
ctx.register_all_quadbin()
ctx.register_rastertile()
ctx.register_raquet(
    "solar",
   "{}spain_solar_ghi.parquet".format(RAQUET_DATA_HOME_DIR),
)

ctx.register_raquet(
    "tci",
    "{}tci_interleaved_gzip.parquet".format(RAQUET_DATA_HOME_DIR),
)

sql = """
LOAD raquet;
"""
duckdb.sql(sql)


@timer
def datafusion_pixel_count():
    sql = """
    with data as (
    SELECT block,native_tile(band_1) as native from solar where block<>0 
    limit 1500
    ),
    out as ( select array_length(native,1) l from data)

    select sum(l) total_pixels from out
    """
    # decoded = ctx.sql(sql)
    ctx.sql(sql).show()

@timer
def duckdb_pixel_count():

    sql = """
   
    with data as (   
    select block,raquet_decode_band(band_1,'float32',256,256,'gzip') as pixel_value 
    FROM read_raquet('/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet')
    limit 1500

    ),
    out as ( select length(pixel_value) l from data)
    select sum(l) total_pixels from out

    """
    duckdb.sql(sql).show()


@timer
def datafusion_raster_stats():
    sql = """
    with data as (
    SELECT block,statistics_tile(band_1) as stats from solar where block<>0    
   
    ),
     out as (select block,unnest(stats) l from data)

    select count(*) from out
    """
    # decoded = ctx.sql(sql)
    ctx.sql(sql).show()

@timer
def duckdb_raster_stats():

    sql = """
   
    with data as (   
    select block,ST_RasterSummaryStats(band_1,'float32',256,256,'gzip') as stats 
    FROM read_raquet('/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet')

    ),
    out as ( select block,unnest(stats) l from data)
    --select sum(l) total_pixels from out
    select count(*) from out

    """
    duckdb.sql(sql).show()



# duckdb_pixel_count()
# datafusion_pixel_count()


# duckdb_raster_stats()
datafusion_raster_stats()
