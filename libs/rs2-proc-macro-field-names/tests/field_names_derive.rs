use rs2_proc_macro_field_names::FieldNames;

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(FieldNames)]
	struct MyStruct {
		field1: i32,
		field2: String,
		field3: f64,
	}

	#[test]
	fn test_field_names_enum() {
		// Ensure the enum is generated with the correct name and variants
		let field = MyStructFields::field1;
		match field {
			MyStructFields::field1 => {}
			_ => panic!("Expected MyStructFields::field1"),
		}

		let field = MyStructFields::field2;
		match field {
			MyStructFields::field2 => {}
			_ => panic!("Expected MyStructFields::field2"),
		}

		let field = MyStructFields::field3;
		match field {
			MyStructFields::field3 => {}
			_ => panic!("Expected MyStructFields::field3"),
		}
	}
}