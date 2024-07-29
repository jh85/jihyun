mod apriori;
mod fpgrowth;

use std::fs::File;
use serde_json;
use std::io::BufReader;

use rand::Rng;
use rand::seq::SliceRandom;
use std::time::Instant;

use fp_growth::algorithm::FPGrowth;

fn generate_dataset(dataset_size: usize,
                    itemset_max: usize,
                    itemset_len: usize) -> Vec<Vec<i32>> {
    let mut rng = rand::thread_rng();
    let mut dataset = Vec::with_capacity(dataset_size);

    for _ in 0..dataset_size {
        let size = rng.gen_range(1 ..= itemset_len);
        let mut transaction: Vec<i32> = (1 ..= itemset_max as i32).collect();
        transaction.shuffle(&mut rng);
        transaction.truncate(size);
        transaction.sort();
        dataset.push(transaction);
    }
    dataset
}

fn main() {
    let mut k = 1;
    loop {
        // dataset-size,  item-value-max,   itemset-max-length
        let dataset = generate_dataset(28816,100,24);

        // let file = File::open("olii.json").unwrap();
        // let reader = BufReader::new(file);
        // let dataset: Vec<Vec<i32>> = serde_json::from_reader(reader).unwrap();
        // println!("{}", dataset.len());

        let min_support_count = 50 - k;

        let t1 = Instant::now();
        let frequent_itemsets1 = apriori::apriori(&dataset, min_support_count);
        let d1 = t1.elapsed();

        let t2 = Instant::now();
        let frequent_itemsets2 = fpgrowth::fpgrowth(&dataset, min_support_count);
        let d2 = t2.elapsed();

        let t3 = Instant::now();
        let frequent_itemsets3 = FPGrowth::<i32>::new(dataset, min_support_count).find_frequent_patterns();
        let d3 = t3.elapsed();

        println!("Result: support count: {}", min_support_count);
        println!("  Apriori   count: {} time:{:?}", frequent_itemsets1.len(), d1);
        println!("  FP-Growth count: {} time:{:?}", frequent_itemsets2.len(), d2);
        println!("  fp-growth count: {} time:{:?}", frequent_itemsets3.frequent_patterns_num(), d3);

        k += 1;
    }
}
