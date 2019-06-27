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

fn total_function(_:Python,clicks:Vec<(i64,i64,f64,f64,i8,i64)>,historic :Vec<(i64,(Vec<f64>,Vec<f64>))>)-> PyResult<Vec<(i64,i64,usize,usize,usize,usize,usize)>> {
    let mut historic_hash = HashMap::new();
    historic_hash.extend(historic);
    let mut accumulator = vec![];
    let splitted = split_vector(&clicks);
    for same_date in splitted {
        let out = historic_distances(&same_date,&historic_hash);
        accumulator.push(out);
        add_to_historic(&same_date,&mut historic_hash);
    }
   Ok(accumulator.into_iter().flat_map(|s| s).collect())
}

fn haversine_py(_:Python,clicks:Vec<(i64,i64,f64,f64,i8,i64)>,historic :Vec<(i64,(Vec<f64>,Vec<f64>))>)-> PyResult<Vec<(i64,i64,usize,usize,usize,usize,usize)>> {
    let mut historic_hash = HashMap::new();
    historic_hash.extend(historic);
    let out = historic_distances(&clicks,&historic_hash); 
    Ok(out)
}

fn add_to_historic(clicks: &Vec<(i64,i64,f64,f64,i8,i64)>, historic : &mut HashMap<i64,(Vec<f64>,Vec<f64>)>) {
    for row in clicks.iter() {
        if row.4 == 1 {
            let person = row.0;
            let hist_reactions = historic.entry(person).or_insert((vec![],vec![]));
            hist_reactions.0.push(row.2);
            hist_reactions.1.push(row.3);
        }
    }
}

// this is very much work in progress!!!!!
fn split_vector(clicks: &Vec<(i64,i64,f64,f64,i8,i64)>) -> Vec<Vec<(i64,i64,f64,f64,i8,i64)>> {
    let mut splitted = vec![]; 
    let mut datum : i64 = 0;
    for row in clicks{
        if row.5 > datum {
            datum = row.5;
            splitted.push(vec![]);
        }
        splitted.last_mut().unwrap().push(row.clone());
    }
    return splitted
}



fn calculate_list_distance(lon:f64,lat:f64,lon_hist:&Vec<f64>,lat_hist:&Vec<f64>) -> Vec<usize> {
    let mut distances = vec![];
    for (&lon_2,&lat_2) in lon_hist.iter().zip(lat_hist.iter()) {
        distances.push(calculate_haversine(lon,lat,lon_2,lat_2));
    }
    let mut within_radius = vec![];
    for radius in [0.5,2.0,5.0,10.0].iter() {
        within_radius.push(distances.iter().filter(|&x| x < radius).count())
    }
    within_radius.push(distances.len());
    within_radius 
}

fn historic_distances(clicks:&Vec<(i64,i64,f64,f64,i8,i64)>,historic :&HashMap<i64,(Vec<f64>,Vec<f64>)>) -> Vec<(i64,i64,usize,usize,usize,usize,usize)> {
    let mut distances = vec![];
    for click in clicks {
        match historic.get(&click.0) {
            Some(coords) => {
                let dist_list = calculate_list_distance(
                            click.2,
                            click.3,
                            &coords.0,
                            &coords.1
                );
 
                distances.push(
                    (
                        click.0,
                        click.1,
                        dist_list[0],
                        dist_list[1],
                        dist_list[2],
                        dist_list[3],
                        dist_list[4]
                    ) 
                ) 
            },
            None => {
                distances.push(
                    (
                        click.0,
                        click.1,
                        0,
                        0,
                        0,
                        0,
                        0,
                    )
                )
            }
        };
    };
    distances 
}

py_module_initializer!(libmyrustlib, initlibmyrustlib, PyInit_myrustlib, |py, m | {
    try!(m.add(py, "__doc__", "This module is implemented in Rust"));
    m.add(py, "haversine", py_fn!(py, haversine_py(clicks:Vec<(i64,i64,f64,f64,i8,i64)>,historic :Vec<(i64,(Vec<f64>,Vec<f64>))>)))?;
    m.add(py, "total_function", py_fn!(py, total_function(clicks:Vec<(i64,i64,f64,f64,i8,i64)>,historic :Vec<(i64,(Vec<f64>,Vec<f64>))>)))?;
    Ok(())
});
