use std::path::PathBuf;

use csv::Reader;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use rand::distributions::{Distribution, Uniform};
use serde::Deserialize;
use structopt::StructOpt;

#[derive(Deserialize)]
pub struct City {
    x: f64,
    y: f64,
}

impl City {
    pub fn new(x: f64, y: f64) -> City {
        City { x, y }
    }
}

#[derive(Clone)]
pub struct Path {
    fitness: f64,
    order: Vec<usize>
}

impl Path {
    pub fn breed(&self, other: &Path, city_list: &Vec<City>) -> Path {
        let order = Path::crossover(&self.order, &other.order);
        let fitness = Path::calculate_fitness(&order, city_list);

        Path { fitness, order }
    }

    fn crossover(mother: &Vec<usize>, father: &Vec<usize>) -> Vec<usize> {
        let mut rng = thread_rng();
        let crossover_point = Uniform::new(0, mother.len()).sample(&mut rng);

        let mother_dna = &mother[0..crossover_point];
        let mut father_dna: Vec<usize> = father.iter().filter_map(|d| {
            if !mother_dna.contains(d) {
                return Some(*d)
            }
            None
        }).collect();

        let mut child = Vec::new();
        child.extend_from_slice(mother_dna);
        child.append(&mut father_dna);

        child
    }

    pub fn mutate(&mut self, city_list: &Vec<City>) {
        let mut rng = thread_rng();
        let point_one = Uniform::new(0, self.order.len()).sample(&mut rng);
        let point_two = Uniform::new(0, self.order.len()).sample(&mut rng);

        self.order.swap(point_one, point_two);
        self.fitness = Path::calculate_fitness(&self.order, &city_list);
    }

    pub fn calculate_fitness(path: &Vec<usize>, city_list: &Vec<City>) -> f64 {
        let path_length = city_list.len();
        let mut cost = 0.0;
        for i in 0..path_length - 1 {
            let a = &city_list[path[i]];
            let b = &city_list[path[i + 1]];
            cost = cost + ((a.x - b.x).powf(2.0) + (a.y - b.y).powf(2.0)).sqrt();
        }

        1.0 / cost
    }
}

pub struct Simulation {
    population: Vec<Path>,
    city_list: Vec<City>,
    max_iterations: usize,
    crossover_rate: f64,
    mutation_rate: f64,
    survival_rate: f64,
}

impl Simulation {
    pub fn new(
        population_size: usize,
        cities: Vec<City>,
        max_iterations: usize,
        crossover_rate: f64,
        mutation_rate: f64,
        survival_rate: f64,
    ) -> Simulation {
        Simulation {
            population: Simulation::initial_population(&cities, population_size),
            city_list: cities,
            max_iterations,
            crossover_rate,
            mutation_rate,
            survival_rate,
        }
    }

    pub fn run(&mut self) -> () {
        let mut fittest = self.find_fittest();
        println!("starting iterations");

        for _ in 0..self.max_iterations {
            self.generate_next_generation();

            let challenger = self.find_fittest();
            if challenger.fitness > fittest.fitness {
                fittest = challenger;
            }
        }

        let order: Vec<String> = fittest.order.iter().map(|o| o.to_string()).collect();

        println!("Solution:");
        println!("Fitness {}", fittest.fitness);
        println!("{}", order.join("->"));
    }

    fn find_fittest(&self) -> Path {
        let mut fittest = &self.population[0];

        for i in 1..self.population.len() {
            let p = &self.population[i];
            if p.fitness > fittest.fitness {
                fittest = p;
            }
        }

        return fittest.clone();
    }

    fn generate_next_generation(&mut self) {
        self.population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        let breeding_count = (self.population.len() as f64 * self.crossover_rate) as usize;
        let surviving_parent_count = (breeding_count as f64 * self.survival_rate) as usize;
        let surviving_weak_count = 2;

        let mut breeding_population = Vec::new();
        breeding_population.extend_from_slice(&self.population[0..breeding_count]);

        let mut offspring = Vec::new();
        let mut rng = thread_rng();
        let pcnt_range = Uniform::new(0, breeding_population.len());

        for i in 0..(self.population.len() - surviving_parent_count - surviving_weak_count) {
            let rs = pcnt_range.sample(&mut rng);
            offspring.push(
                breeding_population[i % breeding_population.len()].breed(
                    &breeding_population[rs],
                    &self.city_list
                )
            );
        }

        let mut next_generation = Vec::new();
        next_generation.extend_from_slice(&self.population[0..surviving_parent_count]);
        next_generation.append(&mut offspring);
        // Add a few weak units to keep the genetic diversity
        next_generation.extend_from_slice(
            &self.population[(self.population.len() - surviving_weak_count)..self.population.len()]
        );

        for p in 0..next_generation.len() {
            if thread_rng().gen_bool(self.mutation_rate) {
                next_generation[p].mutate(&self.city_list);
            }
        }

        self.population = next_generation;
    }

    fn initial_population(city_list: &Vec<City>, population_count: usize) -> Vec<Path> {
        let base_list: Vec<usize> = (0..city_list.len()).collect();
        let mut population = Vec::new();

        for _ in 0..population_count {
            let mut p = base_list.clone();
            let mut rng = thread_rng();
            p.shuffle(&mut rng);
            let fitness = Path::calculate_fitness(&p, city_list);

            population.push(Path { fitness, order: p });
        }

        population
    }
}

#[derive(StructOpt)]
#[structopt()]
struct Opt {
    #[structopt(name = "iterations")]
    iterations: usize,
    #[structopt(name = "pop_size")]
    population_size: usize,
    #[structopt(name = "crossover_rate")]
    crossover_rate: f64,
    #[structopt(name = "mutation_rate")]
    mutation_rate: f64,
    #[structopt(name = "survival_rate")]
    survival_rate: f64,
    #[structopt(name = "csv", parse(from_os_str))]
    csv: PathBuf,
}

fn main() {
    let opts = Opt::from_args();
    let mut reader = Reader::from_path(opts.csv).unwrap();
    let cities: Vec<City> = reader.deserialize()
        .map(|r| {
            let result: City = r.unwrap();
            result
        })
        .collect();

    let mut sim = Simulation::new(
        opts.iterations,
        cities,
        opts.population_size,
        opts.crossover_rate,
        opts.mutation_rate,
        opts.survival_rate,
    );
    sim.run();
}
