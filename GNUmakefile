# Nuke built-in rules and variables.
MAKEFLAGS += -rR
.SUFFIXES:

# Convenience macro to reliably declare user overridable variables.
override USER_VARIABLE = $(if $(filter $(origin $(1)),default undefined),$(eval override $(1) := $(2)))

# Target architecture to build for. Default to x86_64.
$(call USER_VARIABLE,KARCH,x86_64)

# Default user QEMU flags. These are appended to the QEMU command calls.
$(call USER_VARIABLE,QEMUFLAGS,-m 2G)

override IMAGE_NAME := template-$(KARCH)

.PHONY: all
all: $(IMAGE_NAME).iso

.PHONY: run-x86_64
run-x86_64: $(IMAGE_NAME).iso
	qemu-system-$(KARCH) \
		-bios OVMF.fd \
		-cdrom $(IMAGE_NAME).iso \
		$(QEMUFLAGS)

limine/limine:
	$(MAKE) -C limine

.PHONY: NoobOS
NoobOS:
	$(MAKE) -C noob-os

$(IMAGE_NAME).iso: limine/limine NoobOS
	rm -rf iso_root
	mkdir -p iso_root/boot
	cp -v noob-os/NoobOS iso_root/boot/
	cp zap-light16.psf iso_root/boot/
	mkdir -p iso_root/boot/limine
	cp -v limine.conf iso_root/boot/limine/
	mkdir -p iso_root/EFI/BOOT
	cp -v limine/limine-bios.sys limine/limine-bios-cd.bin limine/limine-uefi-cd.bin iso_root/boot/limine/
	cp -v limine/BOOTX64.EFI iso_root/EFI/BOOT/
	cp -v limine/BOOTIA32.EFI iso_root/EFI/BOOT/
	xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o $(IMAGE_NAME).iso
	./limine/limine bios-install $(IMAGE_NAME).iso
	rm -rf iso_root

.PHONY: clean
clean:
	$(MAKE) -C noob-os clean
	$(MAKE) -C limine clean
	rm -rf iso_root $(IMAGE_NAME).iso

.PHONY: distclean
distclean: clean
	$(MAKE) -C noob-os distclean
