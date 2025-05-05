CC = clang
ARGS = -xc \
	   -std=gnu99 \
	   -ggdb3 \
	   -Weverything \
	   -Werror \
	   -Wno-used-but-marked-unused \
	   -Wno-padded \
	   -Wno-declaration-after-statement \
	   -Wno-covered-switch-default \
	   -Wno-unsafe-buffer-usage \
	   -Wno-missing-prototypes \
	   -Wno-disabled-macro-expansion

out:
	mkdir ./out

main.o: out main.c
	$(CC) $(ARGS) -c main.c -o out/main.o

main: main.o
	$(CC) out/main.o -o out/main -ldbus-1

.PHONY: run
run: main
	out/main

.PHONY: clean
clean:
	rm -rf out/
