/* Application memory configuration for STM32H753VI */
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* Application starts at ACTIVE region in Bank 1 */
  BOOTLOADER                        : ORIGIN = 0x08000000, LENGTH = 128K   /* Bootloader */
  BOOTLOADER_STATE                  : ORIGIN = 0x08020000, LENGTH = 128K   /* State */
  FLASH                             : ORIGIN = 0x08040000, LENGTH = 768K   /* Rest of Bank 1 for app */
  DFU                               : ORIGIN = 0x08100000, LENGTH = 1024K  /* Entire Bank 2 for DFU */
  RAM                         (rwx) : ORIGIN = 0x24000000, LENGTH = 512K
}

/* Bootloader symbols - state and active relative to FLASH origin */
__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(BOOTLOADER);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(BOOTLOADER);

/* DFU symbols - for dual-bank, relative to DFU origin (Bank 2) */
__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(DFU);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(DFU);
