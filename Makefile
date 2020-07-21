TARGET = test.bin
OBJS = crt0.o test.o
INCLUDE = ./i386-elf-gcc/include

CC = i386-elf-gcc
LD = i386-elf-ld
AS = nasm
CFLAGS += -nostdlib -fno-asynchronous-unwind-tables \
	-I$(INCLUDE) -g -fno-stack-protector
LDFLAGS += --entry=start --oformat=binary -Ttext 0x7c00

.PHONY: all
all :
	make $(TARGET)

%.o : %.c Makefile
	$(CC) $(CFLAGS) -c $<

%.o : %.asm Makefile
	$(AS) -f elf $<

$(TARGET) : $(OBJS) Makefile
	$(LD) $(LDFLAGS) -o $@ $(OBJS)