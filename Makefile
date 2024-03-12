TARGET = kompilator

all:
	cargo build -r
	cp target/release/$(TARGET) .

