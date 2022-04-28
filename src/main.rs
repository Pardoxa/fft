use {
    std::{
        fs::File,
        io::{Write, BufWriter, BufReader, BufRead},
        env
    },
    realfft::{RealFftPlanner}
};

fn print_help() -> !
{
    println!("USAGE:");
    println!("./fft sampling_period colum_of_data input_file_name");
    println!("Note: Column count starts at zero");
    println!("sampling_period is the time interval between two successive data points");
    std::process::exit(1);
}

fn main() {

    let mut args = env::args();
    let sample_interval: f64 = match args.nth(1){
        None => print_help(),
        Some(v) => v.parse().unwrap()
    };
    let index: usize = match args.next()
    {
        Some(v) => v.parse().unwrap(),
        None => print_help()
    };
    let input_file_name = match args.next(){
        None => print_help(),
        Some(v) => v
    };

    let file = File::open(&input_file_name).unwrap();
    let reader = BufReader::new(file);

    let mut vals: Vec<_> = reader.lines()
        .filter_map(
            |v| 
            {
                match v
                {
                    Err(_) => None,
                    Ok(s) => {
                        if s.starts_with('#') {
                            None
                        } else {
                            Some(s)
                        }
                    }
                }
            }
        )
        .map(
            |line|
            {
                let mut iter = line.split_whitespace();
                let val: f64 = iter.nth(index).unwrap().parse().unwrap();
                val
            }
        ).collect();

    let mut planner = RealFftPlanner::new();
    let fft = planner.plan_fft_forward(vals.len());
    let mut spectrum = fft.make_output_vec();
    let norm = 1.0/(vals.len() as f64).sqrt();

    
    fft.process(&mut vals, &mut spectrum).unwrap();
    spectrum.iter_mut()
        .for_each(|v| *v *= norm);

    let name = format!("{input_file_name}.spectrum");
    println!("calculating spectrum, storing in: {name}");
    let test_file = File::create(&name).unwrap();
    let mut input_buf = BufWriter::new(test_file);
    writeln!(input_buf, "# frequency_in_Hz factor?").unwrap();

    let factor = 1.0/(sample_interval* vals.len() as f64);
    
    for (index, v) in spectrum.iter().enumerate()
    {
        let hz = index as f64 * factor;
        writeln!(input_buf, "{hz} {}", v.norm()).unwrap()
    }

    let inverse = planner.plan_fft_inverse(vals.len());
    inverse.process(&mut spectrum, &mut vals).unwrap();
    vals.iter_mut()
        .for_each(|v| *v *= norm);

    let name = format!("{input_file_name}.inverse");
    println!("calculating inverse fourie transform. This should match the original signal. It is stored in {name}");

    let test_file = File::create(name).unwrap();
    let mut input_buf = BufWriter::new(test_file);

    for (t, v) in vals.iter().enumerate()
    {
        writeln!(input_buf, "{} {v}", (1+t) as f64 * sample_interval).unwrap()
    }
}
