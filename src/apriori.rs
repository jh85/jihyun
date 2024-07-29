use std::collections::{HashMap, HashSet};
use itertools::Itertools;

fn generate_l1(dataset: &Vec<Vec<i32>>, min_support_count: usize) -> HashMap<Vec<i32>, HashSet<usize>> {
    let mut l1: HashMap<Vec<i32>, HashSet<usize>> = HashMap::new();
    
    for (tid, transaction) in dataset.iter().enumerate() {
        for &item in transaction {
            l1.entry(vec![item]).or_insert_with(HashSet::new).insert(tid);
        }
    }
    
    l1.into_iter()
        .filter(|(_, tids)| tids.len() >= min_support_count)
        .collect()
}

fn has_infrequent_subset(candidate: &Vec<i32>, prev_l: &HashMap<Vec<i32>, HashSet<usize>>) -> bool {
    candidate.iter().combinations(candidate.len() - 1).any(|subset| {
        !prev_l.contains_key(&subset.into_iter().cloned().collect::<Vec<_>>())
    })
}

fn apriori_gen(prev_l: &HashMap<Vec<i32>, HashSet<usize>>) -> HashMap<Vec<i32>, HashSet<usize>> {
    let mut ck = HashMap::new();
    
    for (item1, tids1) in prev_l.iter() {
        for (item2, tids2) in prev_l.iter() {
            if item1[..item1.len() - 1] == item2[..item2.len() - 1] && item1.last() < item2.last() {
                let mut new_item = item1.clone();
                new_item.push(*item2.last().unwrap());
                
                if !has_infrequent_subset(&new_item, prev_l) {
                    let new_tids: HashSet<_> = tids1.intersection(tids2).cloned().collect();
                    ck.insert(new_item, new_tids);
                }
            }
        }
    }
    
    ck
}

fn gen_lk(ck: HashMap<Vec<i32>, HashSet<usize>>,
	  min_support_count: usize) -> HashMap<Vec<i32>, HashSet<usize>> {
    ck.into_iter()
        .filter(|(_, tids)| tids.len() >= min_support_count)
        .collect()
}

#[allow(dead_code)]
fn apriori_orig(dataset: &Vec<Vec<i32>>, min_support_count: usize)
		-> HashMap<usize, HashMap<Vec<i32>, HashSet<usize>>> {
    let mut l = HashMap::new();
    let l1 = generate_l1(dataset, min_support_count);
    l.insert(1, l1);
    
    let mut k = 2;
    loop {
        if l[&(k-1)].len() < 2 {
            break;
        }
        let ck = apriori_gen(&l[&(k-1)]);
        let lk = gen_lk(ck, min_support_count);
        if lk.is_empty() {
            break;
        }
        l.insert(k, lk);
	k += 1;
    }

    l
}

pub fn apriori(dataset: &Vec<Vec<i32>>, min_support_count: usize)
	       -> HashMap<Vec<i32>, i32> {
    let mut l: HashMap<usize,HashMap<Vec<i32>,HashSet<usize>>> = HashMap::new();
    let l1 = generate_l1(dataset, min_support_count);
    l.insert(1, l1);
    
    let mut k = 2;
    loop {
        if l[&(k-1)].len() < 2 {
            break;
        }
        let ck = apriori_gen(&l[&(k-1)]);
        let lk = gen_lk(ck, min_support_count);
        if lk.is_empty() {
            break;
        }
        l.insert(k, lk);
	k += 1;
    }

    l.into_iter()
	.flat_map(|(_,inner_map)| inner_map.into_iter())
	.map(|(itemset,transactions)| (itemset, transactions.len() as i32))
	.collect()
}


