use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use itertools::Itertools;

type Link = Rc<RefCell<FPNode>>;
type Wink = Weak<RefCell<FPNode>>;

#[derive(Clone)]
struct FPNode {
    value: Option<i32>,
    count: Option<i32>,
    parent: Option<Wink>,
    link: Option<Wink>,
    children: Vec<Link>,
}

impl FPNode {
    fn new(value: Option<i32>, count: Option<i32>, parent: Option<Wink>) -> Link {
        Rc::new(RefCell::new(FPNode {
            value: value,
            count: count,
            parent: parent,
            link: None,
            children: Vec::new(),
        }))
    }
}

struct FPTree {
    frequent: HashMap<i32, i32>,
    headers: HashMap<i32, Option<Wink>>,
    root: Link,
}

impl FPTree {
    fn new(transactions: &Vec<Vec<i32>>,
	   min_support_count: i32,
	   root_value: Option<i32>,
	   root_count: Option<i32>) -> Self {
        let frequent = Self::find_frequent_items(transactions, min_support_count);
        let mut headers = Self::build_header_table(&frequent);
        let root = Self::build_fptree(transactions, root_value, root_count, &frequent, &mut headers);
        
        FPTree {
            frequent,
            headers,
            root,
        }
    }

    fn find_frequent_items(transactions: &Vec<Vec<i32>>,
			   min_support_count: i32) -> HashMap<i32, i32> {
        let mut items: HashMap<i32, i32> = HashMap::new();
        for t in transactions {
            for &item in t {
                *items.entry(item).or_insert(0) += 1;
            }
        }
        items.retain(|_, &mut v| v >= min_support_count);
        items
    }

    fn build_header_table(frequent: &HashMap<i32, i32>) -> HashMap<i32, Option<Wink>> {
        frequent.keys().map(|&k| (k, None)).collect()
    }

    fn build_fptree(transactions: &Vec<Vec<i32>>,
		    root_value: Option<i32>,
		    root_count: Option<i32>, 
                    frequent: &HashMap<i32, i32>,
		    headers: &mut HashMap<i32, Option<Wink>>) -> Link {
        let root = FPNode::new(root_value, root_count, None);
        for t in transactions {
            let mut freq_items: Vec<i32> = t.iter().filter(|&&item| frequent.contains_key(&item)).cloned().collect();
            freq_items.sort_by(|a, b| frequent[b].cmp(&frequent[a]));
            if !freq_items.is_empty() {
                Self::insert_tree(&freq_items, root.clone(), headers);
            }
        }
        root
    }

    fn insert_tree(items: &[i32],
		   node: Link,
		   headers: &mut HashMap<i32, Option<Wink>>) {
	if let Some(&first) = items.first() {
            let child = node.borrow().children.iter().find(|n| n.borrow().value == Some(first)).cloned();

	    let child = if let Some(existing_child) = child {
		// existing_child.borrow_mut().count = existing_child.borrow().count.map(|c| c+1);
		let incremented_count: Option<i32> = existing_child.borrow().count.map(|c| c+1);
		existing_child.borrow_mut().count = incremented_count;
		existing_child
	    } else {
		let new_child = Rc::new(RefCell::new(FPNode {
		    value: Some(first),
		    count: Some(1),
		    parent: Some(Rc::downgrade(&node)),
		    link: None,
		    children: Vec::new(),
		}));
		node.borrow_mut().children.push(new_child.clone());

		if let Some(header) = headers.get_mut(&first) {
		    if header.is_none() {
			*header = Some(Rc::downgrade(&new_child));
		    } else {
			let mut current = header.clone();
			while let Some(current_weak) = current {
			    if let Some(current_node) = current_weak.upgrade() {
				if current_node.borrow().link.is_none() {
				    current_node.borrow_mut().link = Some(Rc::downgrade(&new_child));
				    break;
				}
				current = current_node.borrow().link.clone();
			    } else {
				break;
			    }
			}
		    }
		}
		new_child
	    };

            if items.len() > 1 {
                Self::insert_tree(&items[1..], child, headers);
            }
        }
    }

    fn tree_has_single_path(&self, node: Link) -> bool {
        let node_ref = node.borrow();
        match node_ref.children.len() {
            0 => true,
            1 => self.tree_has_single_path(node_ref.children[0].clone()),
            _ => false,
        }
    }

    fn mine_patterns(&self, min_support_count: i32) -> HashMap<Vec<i32>, i32> {
        if self.tree_has_single_path(self.root.clone()) {
            self.generate_pattern_list()
        } else {
	    let mut patterns = self.zip_patterns(self.mine_sub_trees(min_support_count));
	    if let Some(root_value) = self.root.borrow().value {
		if let Some(root_count) = self.root.borrow().count {
		    patterns.insert(vec![root_value], root_count);
		}
	    }
	    patterns
	}
    }

    fn zip_patterns(&self, patterns: HashMap<Vec<i32>,i32>) -> HashMap<Vec<i32>,i32> {
	if let Some(suffix) = self.root.borrow().value {
	    patterns.into_iter().map(|(mut k,v)| {
		k.push(suffix);
		k.sort();
		(k,v)
	    }).collect()
	} else {
	    patterns
	}
    }

    fn generate_pattern_list(&self) -> HashMap<Vec<i32>, i32> {
	let mut patterns: HashMap<Vec<i32>,i32> = HashMap::new();
	let items: Vec<i32> = self.frequent.keys().cloned().collect();
	let suffix_value = if let Some(root_value) = self.root.borrow().value {
	    let suffix = vec![root_value];
	    if let Some(root_count) = self.root.borrow().count {
		patterns.insert(suffix.clone(), root_count);
	    }
	    suffix
	} else {
	    Vec::new()
	};

        for i in 1 ..= items.len() {
            for subset in items.iter().cloned().combinations(i) {
                let mut pattern: Vec<i32> = subset.clone();
                pattern.extend(&suffix_value);
                pattern.sort();
                let count =subset.iter()
		    .filter_map(|&x| self.frequent.get(&x))
		    .min()
		    .cloned()
		    .unwrap_or(0);
                patterns.insert(pattern, count);
            }
        }
        patterns
    }

    fn mine_sub_trees(&self, min_support_count: i32) -> HashMap<Vec<i32>, i32> {
        let mut patterns: HashMap<Vec<i32>,i32> = HashMap::new();
        let mut mining_order: Vec<i32> = self.frequent.keys().cloned().collect();
        mining_order.sort_by_key(|&x| self.frequent[&x]);

        for &item in &mining_order {
            let mut suffixes: Vec<Link> = Vec::new();
            let mut conditional_tree_input: Vec<Vec<i32>> = Vec::new();

	    let mut weak_node_opt = self.headers.get(&item).and_then(|w| w.as_ref()).cloned();
	    while let Some(weak_node) = weak_node_opt {
		if let Some(node) = weak_node.upgrade() {
		    suffixes.push(node.clone());
		    weak_node_opt = node.borrow().link.as_ref().cloned();
		} else {
		    break;
		}
	    }

            for suffix in suffixes {
                let frequency = suffix.borrow().count.unwrap_or(0);
                let mut path: Vec<i32> = Vec::new();
                let mut weak_parent_opt = suffix.borrow().parent.as_ref().cloned();

                while let Some(weak_parent) = weak_parent_opt {
		    if let Some(parent) = weak_parent.upgrade() {
			if parent.borrow().parent.is_some() {
                            if let Some(value) = parent.borrow().value {
				path.push(value);
                            }
			}
			weak_parent_opt = parent.borrow().parent.clone();
                    } else {
			break;
		    }
		}
		path.reverse();
		conditional_tree_input.extend(std::iter::repeat(path).take(frequency as usize));
            }

            let subtree = FPTree::new(&conditional_tree_input,
				      min_support_count,
				      Some(item),
				      self.frequent.get(&item).cloned());
            let subtree_patterns = subtree.mine_patterns(min_support_count);

            for (mut pattern, count) in subtree_patterns {
                pattern.sort();
                *patterns.entry(pattern).or_insert(0) += count;
            }
        }
        patterns
    }
}

pub fn fpgrowth(transactions: &Vec<Vec<i32>>,
		min_support_count: usize) -> HashMap<Vec<i32>, i32> {
    let transactions_sorted: Vec<Vec<i32>> = transactions
	.iter()
	.map(|t| {
	    let mut sorted_t = t.clone();
	    sorted_t.sort();
	    sorted_t
	})
	.collect();
    let tree = FPTree::new(&transactions_sorted, min_support_count as i32, None, None);
    tree.mine_patterns(min_support_count as i32)
}
