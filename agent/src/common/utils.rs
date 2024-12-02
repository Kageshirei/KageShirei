use alloc::{collections::BTreeSet, string::String, vec::Vec};

use rand::{distributions::Alphanumeric, thread_rng, Rng as _};

/// Generates a random request ID consisting of 32 alphanumeric characters.
///
/// # Returns
/// A `String` containing the generated request ID.
pub fn generate_request_id(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

/// Helper function to generate a random string of length between 3 and 10 characters.
///
/// # Returns
/// A `String` containing the generated random string.
pub fn generate_random_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(thread_rng().gen_range(3 .. 10))
        .map(char::from)
        .collect()
}

/// Generates a unique set of random positions within a specified range.
///
/// # Arguments
///
/// * `num_positions` - The number of unique positions to generate.
/// * `start_index` - The start of the range (inclusive).
/// * `end_index` - The end of the range (exclusive).
///
/// # Returns
///
/// A `Vec<usize>` containing the unique random positions.
pub fn generate_unique_positions(num_positions: usize, start_index: usize, end_index: usize) -> Vec<usize> {
    let mut positions: BTreeSet<usize> = BTreeSet::new();
    // let mut positions = HashSet::new();
    while positions.len() < num_positions {
        positions.insert(thread_rng().gen_range(start_index .. end_index));
    }
    positions.into_iter().collect()
}

/// Generates random lengths that sum to 32
///
/// # Returns
///
/// A `Vec<usize>` containing three random lengths that sum to 32.
pub fn generate_random_lengths_for_request_id(len: usize) -> (usize, usize, usize) {
    let mut rng = thread_rng();
    let first = rng.gen_range(1 .. (len.overflowing_sub(2).0));
    let second = rng.gen_range(1 .. (len.overflowing_sub(first).0));
    let third = len.overflowing_sub(first).0.overflowing_sub(second).0;
    (first, second, third)
}

/// Generates a random path similar to Example 2 in the comments.
///
/// This function randomly generates a path of three types:
/// - Type 0: A single position for the request ID within a range of random strings. The index of
///   the request ID position is included as part of the path.
/// - Type 1: Three positions for fragments of the request ID within a range of random strings,
///   separated by randomly chosen separators. The positions of the fragments are included in the
///   path.
/// - Type 2: The request ID is inserted randomly into the path without any indices or positions
///   being included in the path. The first string of length 32 is automatically recognized as the
///   request ID.
///
/// # Arguments
///
/// * `request_id_len` - The length of the request ID (typically 32).
/// * `start_index` - The start of the range for the ID position(s) (inclusive).
/// * `end_index` - The end of the range for the ID position(s) (exclusive).
///
/// # Returns
///
/// A tuple containing:
/// * The path type (0, 1, or 2).
/// * A `String` containing the generated path.
/// * A `String` containing the generated request ID.
///
/// # Path Type Explanation:
///
/// **Type 0:**
/// Path: `/1/a/b/request_id/c/d`
/// - "1" indicates the position where the request ID appears (position 1 in this case).
///
/// **Type 1:**
/// Path: `/0;2-4/a/b/part1/c/part2/d/part3`
/// - "0;2-4" indicates the positions where the request ID is split and inserted as fragments
///   ("part1", "part2", "part3").
///
/// **Type 2:**
/// Path: `/a/b/request_id/c/d`
/// - The request ID appears as a string of length 32 somewhere in the path without any numerical
///   indices.
pub fn generate_path(request_id_len: usize, start_index: usize, end_index: usize) -> (usize, String, String) {
    // Randomly choose the path type (0, 1, or 2)
    let path_type = thread_rng().gen_range(0 .. 3);
    // Generate a random request ID of the specified length
    let request_id = generate_request_id(request_id_len);

    if path_type == 0 {
        // Type 0: Single position for the request ID
        // Randomly choose an index within the range for the request ID
        let id_position = thread_rng().gen_range(start_index .. end_index);
        let mut path_parts: Vec<String> = (0 .. end_index.overflowing_sub(start_index).0)
            .map(|_| generate_random_string())
            .collect();
        if let Some(part) = path_parts.get_mut(id_position) {
            *part = request_id.clone();
        }
        let id_position_str = id_position.to_string();
        (
            path_type,
            format!("/{}/{}", id_position_str, path_parts.join("/")),
            request_id,
        )
    }
    else if path_type == 1 {
        // Type 1: Multiple positions for fragments of the request ID
        let separators = [",", ";", ":", ".", "-", "_", " ", "|", "$"];
        let chosen_separators: Vec<&str> = (0 .. 2)
            .map(|_| {
                *separators
                    .get(thread_rng().gen_range(0 .. separators.len()))
                    .unwrap_or(&",")
            })
            .collect();

        let id_positions: Vec<usize> = generate_unique_positions(3, start_index, end_index);
        let mut path_parts: Vec<String> = (0 .. end_index.overflowing_sub(start_index).0)
            .map(|_| generate_random_string())
            .collect();

        let (len1, len2, len3) = generate_random_lengths_for_request_id(request_id_len);
        let id_parts = [
            &request_id[0 .. len1],
            &request_id[len1 .. len1.overflowing_add(len2).0],
            &request_id[len1.overflowing_add(len2).0 .. len1.overflowing_add(len2).0.overflowing_add(len3).0],
        ];

        for (i, &pos) in id_positions.iter().enumerate() {
            if let Some(part) = path_parts.get_mut(pos) {
                *part = id_parts.get(i).unwrap_or(&"").to_string();
            }
        }

        (
            path_type,
            format!(
                "/{}{}{}{}{}/{}",
                id_positions.first().unwrap_or(&0),
                chosen_separators.first().unwrap_or(&","),
                id_positions.get(1).unwrap_or(&0),
                chosen_separators.get(1).unwrap_or(&","),
                id_positions.get(2).unwrap_or(&0),
                path_parts.join("/")
            ),
            request_id,
        )
    }
    else {
        // Type 2: Request ID without any index, just randomly placed in the path
        let id_position = thread_rng().gen_range(start_index .. end_index);
        let mut path_parts: Vec<String> = (0 .. end_index.overflowing_sub(start_index).0)
            .map(|_| generate_random_string())
            .collect();
        if let Some(part) = path_parts.get_mut(id_position) {
            *part = request_id.clone();
        }
        (path_type, format!("/{}", path_parts.join("/")), request_id)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AgentErrors {
    ChangeDirectoryFailed,
    PrintWorkingDirectoryFailed,
    CmdOutputIsEmpty,
    Other(&'static str),
}

impl AgentErrors {
    /// Returns a message describing the error.
    pub const fn message(&self) -> &str {
        match *self {
            Self::ChangeDirectoryFailed => "Failed to change directory",
            Self::PrintWorkingDirectoryFailed => "Failed to retrieve current working directory",
            Self::CmdOutputIsEmpty => "Failed to retrieve output from cmd",
            Self::Other(msg) => msg,
        }
    }
}

/// Downcasts a u128 to an i64 by taking the rightmost 64 bits of the u128.
///
/// # Arguments
///
/// * `value` - The u128 value to downcast.
///
/// # Returns
///
/// The i64 value of the rightmost 64 bits of the u128.
pub const fn downcast_rightmost_u128(value: u128) -> i64 {
    let mask = 0xffff_ffff_ffff_ffff;
    ((value & mask) as i128) as i64
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_generate_request_id() {
        let request_id = generate_request_id(32);
        // libc_println!("Generated Request ID: {}", request_id);
        assert_eq!(request_id.len(), 32, "Request ID length is not 32");
        assert!(
            request_id.chars().all(char::is_alphanumeric),
            "Request ID contains non-alphanumeric characters"
        );
    }

    #[test]
    fn test_generate_path() {
        for _ in 0 .. 10 {
            let request_id_len = 32;
            let start_index = 0;
            let end_index = 6;

            // Generate the path, path type, and request ID
            let (path_type, path, request_id) = generate_path(request_id_len, start_index, end_index);

            // libc_println!("Generated Path: - {}: {}", path_type, path);
            // libc_println!("Request ID: {}", request_id);

            // Split the path into parts by "/"
            let parts: Vec<&str> = path.split('/').collect();

            if path_type == 0 {
                // Check for type 0 path
                assert_eq!(
                    parts.len(),
                    end_index + 2,
                    "Path does not contain the expected number of parts"
                );
                let id_position: usize = parts[1].parse().unwrap();
                // Ensure id_position is within the valid range
                assert!(id_position < end_index, "id_position is out of valid range");
                // Check that the request ID matches the part at the specified position
                assert_eq!(
                    parts[id_position + 2],
                    request_id,
                    "Request ID does not match at the specified position"
                );
            }
            else if path_type == 1 {
                // Check for type 1 path
                assert_eq!(
                    parts.len(),
                    end_index + 2,
                    "Path does not contain the expected number of parts"
                );

                let first_part = parts[1];
                let separators = [",", ";", ":", ".", "-", "_", " ", "|", "$"];
                let mut separator_count = 0;
                let mut id_positions = Vec::new();

                // Count the separators in the first part
                for sep in &separators {
                    if first_part.contains(*sep) {
                        separator_count += first_part.matches(*sep).count();
                    }
                }
                // Ensure there are exactly 2 separators
                assert_eq!(separator_count, 2, "Path does not contain 2 separators");

                // Extract positions from the first part
                let positions_and_separators: Vec<&str> = first_part.split(|c: char| !c.is_numeric()).collect();
                for pos in positions_and_separators {
                    if let Ok(position) = pos.parse::<usize>() {
                        id_positions.push(position);
                    }
                }

                // Ensure there are exactly 3 ID positions
                assert_eq!(
                    id_positions.len(),
                    3,
                    "Path does not contain 3 ID positions"
                );
                // Ensure all ID positions are within the valid range
                assert!(
                    id_positions.iter().all(|&pos| pos < end_index),
                    "One or more ID positions are out of valid range"
                );

                // Concatenate ID fragments from the specified positions
                let mut concatenated_id = String::new();
                for &pos in &id_positions {
                    concatenated_id.push_str(parts[pos + 2]);
                }

                // Ensure the concatenated ID length is 32
                assert_eq!(
                    concatenated_id.len(),
                    32,
                    "Concatenated ID parts length is not 32"
                );
                // Ensure the concatenated ID matches the request ID
                assert_eq!(
                    concatenated_id, request_id,
                    "Concatenated ID does not match the request ID"
                );
            }
            else {
                // Check for type 2 path
                // Ensure that the path contains the correct number of parts
                assert_eq!(
                    parts.len(),
                    end_index + 1,
                    "Path does not contain the expected number of parts"
                );

                // Ensure there is one part with length equal to the request ID (32 characters)
                let mut found_request_id = None;
                for part in &parts {
                    if part.len() == 32 {
                        found_request_id = Some(part.to_string());
                        break;
                    }
                }

                // Ensure we found the request ID in the path
                assert!(
                    found_request_id.is_some(),
                    "Did not find the request ID in the path"
                );

                // Ensure the found request ID matches the generated request ID
                assert_eq!(
                    found_request_id.unwrap(),
                    request_id,
                    "Request ID found in the path does not match the generated request ID"
                );
            }
            // libc_println!();
        }
    }
}
