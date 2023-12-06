all: Disk.img

build: Disk.img

Disk.img:
	cargo run

run:
	qemu-system-x86_64 -L . -m 64 -fda Disk.img -M pc 


clean:
	cargo clean
	rm Disk.img