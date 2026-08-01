#![cfg(feature = "std")]
#![allow(unused)]

use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct HuffmanCode {
    pub len: u16,
    pub code: u16,
}

// Internal node representation utilizing safe, heap-allocated children
#[derive(Clone, Debug)]
enum TreeItem {
    Leaf { symbol: u8 },
    Internal { left: Box<TreeNode>, right: Box<TreeNode> },
}

#[derive(Clone, Debug)]
struct TreeNode {
    weight: u32,
    item: TreeItem,
}

// Implement Ord to prioritize lower weights first inside the BinaryHeap min-heap
impl PartialEq for TreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}
impl Eq for TreeNode {}

impl PartialOrd for TreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TreeNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Tie-breaker: If weights are identical, prioritize leaves over internal nodes
        // to maintain uniform code length properties
        self.weight.cmp(&other.weight).then_with(|| match (&self.item, &other.item) {
            (TreeItem::Leaf { .. }, TreeItem::Internal { .. }) => std::cmp::Ordering::Less,
            (TreeItem::Internal { .. }, TreeItem::Leaf { .. }) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        })
    }
}

/// Generates a left-aligned Huffman code table from an array of 256 byte frequencies.
/// Generates a left-aligned Huffman code table from an array of 256 byte frequencies.
pub fn generate_huffman_table(frequencies: &[u32; 256]) -> [HuffmanCode; 256] {
    let mut table = [HuffmanCode::default(); 256];

    let mut heap = BinaryHeap::new();
    for (symbol, &freq) in frequencies.iter().enumerate() {
        if freq > 0 {
            heap.push(Reverse(TreeNode {
                weight: freq,
                #[allow(clippy::cast_possible_truncation)]
                item: TreeItem::Leaf { symbol: symbol as u8 },
            }));
        }
    }

    // Edge Case: Empty frequency map profile passed
    if heap.is_empty() {
        return table;
    }

    // Edge Case: Only exactly 1 unique symbol exists in data array
    if heap.len() == 1 {
        if let Some(Reverse(root)) = heap.pop()
            && let TreeItem::Leaf { symbol } = root.item
        {
            table[symbol as usize] = HuffmanCode { len: 1, code: 0x0000 };
        }
        return table;
    }

    // 2. Build the tree recursively using safe pattern matching instead of unwrap
    // 2. Build the tree recursively using a clean let...else statement
    while heap.len() > 1 {
        let (Some(Reverse(left)), Some(Reverse(right))) = (heap.pop(), heap.pop()) else {
            break;
        };

        let parent = TreeNode {
            weight: left.weight + right.weight,
            item: TreeItem::Internal { left: Box::new(left), right: Box::new(right) },
        };
        heap.push(Reverse(parent));
    }

    // The single item remaining on the heap represents the root of the tree
    let Some(Reverse(root_node)) = heap.pop() else {
        return table;
    };

    // 3. Iterative DFS traversal to map codes using a standard vector path stack
    let mut dfs_stack = vec![(Box::new(root_node), 0u16, 0u16)];

    while let Some((node, curr_bits, length)) = dfs_stack.pop() {
        match node.item {
            TreeItem::Leaf { symbol } => {
                let left_aligned_code = curr_bits << (16 - length);
                table[symbol as usize] = HuffmanCode { len: length, code: left_aligned_code };
            }
            TreeItem::Internal { left, right } => {
                dfs_stack.push((right, (curr_bits << 1) | 1, length + 1));
                dfs_stack.push((left, curr_bits << 1, length + 1));
            }
        }
    }

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_frequencies() {
        let frequencies = [0u32; 256];
        let table = generate_huffman_table(&frequencies);

        // If no bytes appear, every entry should remain zeroed out
        for code in &table {
            assert_eq!(code.len, 0);
            assert_eq!(code.code, 0);
        }
    }

    #[test]
    fn test_single_unique_character() {
        let mut frequencies = [0u32; 256];
        frequencies[b'A' as usize] = 42; // Only 'A' exists

        let table = generate_huffman_table(&frequencies);

        // Single character should result in a 1-bit code
        assert_eq!(table[b'A' as usize].len, 1);
        assert_eq!(table[b'A' as usize].code, 0x0000); // 0 left-aligned

        // All other entries must remain unused
        for (i, code) in table.iter().enumerate() {
            if i != b'A' as usize {
                assert_eq!(code.len, 0);
            }
        }
    }

    #[test]
    fn test_predictable_distribution() {
        let mut frequencies = [0u32; 256];
        // Set up frequencies where 'A' is highly common, 'B' is moderate, 'C' and 'D' are rare.
        frequencies[b'A' as usize] = 100;
        frequencies[b'B' as usize] = 40;
        frequencies[b'C' as usize] = 10;
        frequencies[b'D' as usize] = 10;

        let table = generate_huffman_table(&frequencies);

        let code_a = table[b'A' as usize];
        let code_b = table[b'B' as usize];
        let code_c = table[b'C' as usize];
        let code_d = table[b'D' as usize];

        // 1. Verify frequency rule: More frequent characters must have shorter or equal lengths
        assert!(code_a.len <= code_b.len);
        assert!(code_b.len <= code_c.len);
        assert_eq!(code_c.len, code_d.len); // C and D have identical weights, so they should match in depth

        // 2. Verify that they are valid lengths
        assert!(code_a.len > 0);
        assert!(code_b.len > 0);
        assert!(code_c.len > 0);
        assert!(code_d.len > 0);
    }

    #[test]
    fn test_left_alignment_invariant() {
        let mut frequencies = [0u32; 256];
        #[allow(clippy::cast_possible_truncation)]
        for (i, frequency) in frequencies.iter_mut().enumerate().take(10) {
            // `i` is your index (0 to 9)
            // `frequency` is a mutable reference (`&mut T`) to the item at frequencies[i]
            *frequency = (i + 1) as u32;
        }

        let table = generate_huffman_table(&frequencies);

        for code in &table {
            if code.len > 0 {
                // In your architecture, code bits are packed to the left side of the u16 container.
                // This means any bits past the assigned length boundary MUST be strictly 0.
                let unused_bits_mask = (1 << (16 - code.len)) - 1;
                assert_eq!(
                    code.code & unused_bits_mask,
                    0,
                    "Found dirty bits in the lower right padding area of code: {code:?}"
                );
            }
        }
    }
}
