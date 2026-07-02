import duckdb
from datafusion import SessionConfig
from pyarrow import Table
from datafusion_raquet import RaquetSessionContext
from dotenv import load_dotenv
from pathlib import Path
import os
import time

# dotenv_path = Path('/home/jonrm/projects/git/datafusion-raquet/.env')
# RAQUET_DATA_HOME_DIR = os.getenv('RAQUET_DATA_HOME_DIR')

def timer(func):
    def wrapper(*args, **kwargs):
        nonlocal total
        start = time.time()
        result = func(*args, **kwargs)
        duration = time.time() - start
        total += duration
        print(f"Execution time: {func.__name__} Execution time: {duration*1000}ms   ")
        return result

    total = 0
    return wrapper

@timer
def duckdb_setup() -> duckdb.DuckDBPyConnection:
    con = duckdb.connect()

    sql = """
  
    INSTALL httpfs;
    INSTALL raquet;
    LOAD httpfs;
    LOAD raquet;
    """
    con.execute(sql)

    return con

@timer
def datafusion_local_setup() -> RaquetSessionContext:

    config = SessionConfig().with_information_schema(True)

    ctx = RaquetSessionContext(config=config)
    ctx.register_all_quadbin()
    ctx.register_rastertile()
    ctx.register_views()
    ctx.register_raquet(
        "solar",
    "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet",
    )
    return ctx




@timer
def datafusion_decode_band(ctx):
    sql = """
    with data as (
    SELECT native_tile(band_1) as decoded from solar where block<>0 
    limit 1000
    ),
    out as ( select array_length(decoded,1) l from data)

    select sum(l) total_pixels from out
    """
    # decoded = ctx.sql(sql)
    ctx.sql(sql).collect()

@timer
def duckdb_decode_band(con:duckdb.DuckDBPyConnection):
    sql = """
    with data as 
    (
    select 
    raquet_decode_band(band_1,'float32',256,256,'gzip') decoded
    from
    read_raquet('/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet') limit 1000
    ),
    out as (
    select  length(decoded) l from data
    )
    select sum(l) from out


    """
    con.execute(sql)


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
def datafusion_raster_stats(ctx):
    sql = """
    with data as (
    SELECT block,statistics_tile(band_1) as stats from solar where block<>0    
   
    ),
     out as (select block,unnest(stats) l from data)

    select * from out
    """
    # decoded = ctx.sql(sql)
    ctx.sql(sql).collect()

@timer
def duckdb_raster_stats(con:duckdb.DuckDBPyConnection):

    sql = """
   
    with data as (   
    select block,ST_RasterSummaryStats(band_1,'float32',256,256,'gzip') as stats 
    FROM read_raquet('/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet')

    ),
    out as ( select block,unnest(stats) l from data)
    
    select * from out

    """
    con.execute(sql)



@timer
def duckdb_read_raquet_metadata():

    sql = """
    --INSTALL httpfs;
    --LOAD httpfs;
    LOAD raquet;
    select * from read_raquet_metadata('https://storage.googleapis.com/raquet_demo_data/spain_solar_ghi.parquet');


   -- block,ST_RasterValue(block, band_1, ST_Point(-3.7038, 40.4168), metadata) AS red
   
   -- FROM read_raquet_at('/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet', -3.7038, 40.4168);

    """
    duckdb.sql(sql).show()

@timer
def duckdb_remote_read_raquet_at():

    sql = """
    select
    ST_RasterValue(block, band_1, ST_Point(-3.7038, 40.4168), metadata) AS red
   
    FROM read_raquet_at('https://storage.googleapis.com/raquet_demo_data/spain_solar_ghi.parquet', -3.7038, 40.4168);

    """
    duckdb.sql(sql).show()

@timer
def duckdb_local_read_raquet_at():

    sql = """
    select
    ST_RasterValue(block, band_1, ST_Point(-3.7038, 40.4168), metadata) AS red
   
    FROM read_raquet_at('/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet', -3.7038, 40.4168);

    """
    duckdb.sql(sql).show()

@timer
def duckdb_local_read_raquet():

    sql = """
    select
    block  
    FROM 
    read_raquet('/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet')
    limit 1
    ;

    """
    duckdb.sql(sql).show()

@timer
def duckdb_remote_read_raquet():

    sql = """
    select
    block
    FROM 
    read_raquet('https://storage.googleapis.com/raquet_demo_data/spain_solar_ghi.parquet')
    limit 1
    ;

    """
    duckdb.sql(sql).show()

@timer
def duckdb_multiple_read_raquet_at():

    sql = """ 
    with test as 
        (
        select unnest(generate_series(1,10,1)) as output
        ),
    result as 
        (
        select 
        test.output,
        rr.block
        FROM read_raquet_at('https://storage.googleapis.com/raquet_demo_data/spain_solar_ghi.parquet', -3.7038, 40.4168) rr, test    
        )

        select * from result

    """
    duckdb.sql(sql).show()

# duckdb_pixel_count()
# datafusion_pixel_count()


ctx = datafusion_local_setup()
con = duckdb_setup()

# datafusion_setup()
# duckdb_multiple_read_raquet_at()
# duckdb_local_read_raquet_at()

# duckdb.sql(sql).show()
# duckdb_read_raquet_at()

# duckdb_raster_stats(con)
# datafusion_raster_stats(ctx)
# duckdb_decode_band(con)
# datafusion_decode_band(ctx)

# duckdb_local_read_raquet()
# duckdb_remote_read_raquet()

sql = """

select block from read_raquet('solar') limit 1;
"""
ctx.sql(sql).collect()
