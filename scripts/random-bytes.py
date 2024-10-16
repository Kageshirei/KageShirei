import sys
import os
import binascii

def generate_random_hex_string(size):
    # Generate a random byte string of the given size
    random_bytes = os.urandom(size)
    # Convert the byte string to a hex string
    hex_string = binascii.hexlify(random_bytes).decode('utf-8')
    return hex_string

def main():
    if len(sys.argv) != 2:
        print("Usage: python script.py <size>")
        sys.exit(1)

    try:
        size = int(sys.argv[1])
        if size <= 0:
            raise ValueError

        hex_string = generate_random_hex_string(size)
        print(f"Random hex string of size {size}: {hex_string}")

    except ValueError:
        print("Please enter a valid positive integer for the size.")

if __name__ == "__main__":
    main()
