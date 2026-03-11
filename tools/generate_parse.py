import argparse
import sys
from collections import defaultdict

def generate_rust_match(enum_name, tags):
    by_len = defaultdict(list)
    for tag in tags:
        if tag.strip():
            by_len[len(tag.strip())].append(tag.strip())

    lines = []
    lines.append(f"impl {enum_name} {{")
    lines.append(f"    pub fn from_str_insensitive(s: &str) -> Option<Self> {{")
    lines.append("        let bytes = s.as_bytes();")
    lines.append("        match bytes.len() {")

    for length in sorted(by_len.keys()):
        lines.append(f"            {length} => {{")
        lines.append("                match [")
        byte_accessors = ", ".join([f"bytes[{i}].to_ascii_lowercase()" for i in range(length)])
        lines.append(f"                    {byte_accessors}")
        lines.append("                ] {")

        for tag in by_len[length]:
            byte_pattern = ", ".join([f"b'{c.lower()}'" for c in tag])
            lines.append(f"                    [{byte_pattern}] => Some({enum_name}::{tag}),")

        lines.append("                    _ => None,")
        lines.append("                }")
        lines.append("            }")

    lines.append("            _ => None,")
    lines.append("        }")
    lines.append("    }")
    lines.append("}")

    return "\n".join(lines)

def main():
    parser = argparse.ArgumentParser(description="Generate case-insensitive Rust parser")
    parser.add_argument("--enum", required=True, help="Name of the Rust Enum")
    parser.add_argument("--file", required=True, help="Path to text file with variants")
    args = parser.parse_args()

    try:
        with open(args.file, 'r') as f:
            tags = [line.strip() for line in f if line.strip()]

        print(generate_rust_match(args.enum, tags))
    except FileNotFoundError:
        print(f"Error: File {args.file} not found.", file=sys.stderr)

if __name__ == "__main__":
    main()
