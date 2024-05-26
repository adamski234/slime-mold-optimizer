fn main() {
	let header = "z_parameter,migration_threshold,fn_name,max_solution,avg_solution,min_solution";

	let filename_gex = regex::Regex::new(r".*_(\d*\.\d*)_.*_(\d*\.\d*)").unwrap();
	let gex = regex::Regex::new(r"(.*): .*is (-?\d*(?:\.\d*)?)\..*is (-?\d*(?:\.\d*)?)\. .*is (-?\d*(?:\.\d*)?)").unwrap();

	println!("{}", header);

	for filename in glob::glob("./output_slime/*").unwrap() {
		let filename = filename.unwrap();
		let filename_captures = filename_gex.captures(filename.to_str().unwrap()).unwrap();
		let stat_data = std::fs::read_to_string(&filename).unwrap().lines().map(|line| {
			let captures = gex.captures(line).unwrap();
			return format!("{},{},{},{},{},{}", &filename_captures[2], &filename_captures[1], &captures[1], &captures[2], &captures[3], &captures[4]);
		}).collect::<Vec<_>>().join("\n");
		println!("{}", stat_data);
	}
}