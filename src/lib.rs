#[macro_use]
extern crate cpython;

use cpython::{Python, PyResult};
use std::collections::HashMap; 

fn calculate_haversine(mut lon : f64,mut lat: f64,mut lon_2: f64,mut lat_2: f64) -> f64 {
    lon = lon.to_radians();
    lat = lat.to_radians();
    lon_2 = lon_2.to_radians();
    lat_2 = lat_2.to_radians();
    let dlon = lon_2 - lon;
    let dlat = lat_2 - lat;
    let a = (dlat /2.0).sin().powi(2) + lat.cos() * lat_2.cos() * (dlon/2.0).sin().powi(2); 
    let c = 2.0 * a.sqrt().asin();
    let r = 6371.0;
    c * r
}

fn haversine_py(_:Python,clicks:Vec<(i64,i64,f64,f64)>,historic :Vec<(i64,(Vec<f64>,Vec<f64>))>)-> PyResult<Vec<(i64,i64,Vec<usize>)>> {
    let mut historic_hash = HashMap::new();
    historic_hash.extend(historic);
    let out = historic_distances(clicks,historic_hash); 
    Ok(out)
}

fn calculate_list_distance(lon:f64,lat:f64,lon_hist:&Vec<f64>,lat_hist:&Vec<f64>) -> Vec<usize> {
    let mut distances = vec![];
    for (&lon_2,&lat_2) in lon_hist.iter().zip(lat_hist.iter()) {
        distances.push(calculate_haversine(lon,lat,lon_2,lat_2));
    }
    let mut within_radius = vec![];
    for radius in [0.1,0.2,0.5,1.0,2.0,5.0,10.0,15.0].iter() {
        within_radius.push(distances.iter().filter(|&x| x < radius).count())
    }
    within_radius.push(distances.len());
    within_radius 
}

fn historic_distances(clicks:Vec<(i64,i64,f64,f64)>,historic :HashMap<i64,(Vec<f64>,Vec<f64>)>) -> Vec<(i64,i64,Vec<usize>)> {
    let mut distances = vec![];
    for click in clicks {
        match historic.get(&click.0) {
            Some(coords) => {
                distances.push(
                    (click.0,
                     click.1,
                    calculate_list_distance(
                            click.2,
                            click.3,
                            &coords.0,
                            &coords.1
                            )
                        ) 
                   ) 
            },
            None => {
                distances.push(
                    (
                        click.0,
                        click.1,
                        vec![0;9]
                    )
                )
            }
        };
    };
    distances 
}

            

py_module_initializer!(libmyrustlib, initlibmyrustlib, PyInit_myrustlib, |py, m | {
    try!(m.add(py, "__doc__", "This module is implemented in Rust"));
    m.add(py, "haversine", py_fn!(py, haversine_py(clicks:Vec<(i64,i64,f64,f64)>,historic :Vec<(i64,(Vec<f64>,Vec<f64>))>)))?;
    Ok(())
});
