/* Bootloader memory configuration for STM32H753VI - Dual Bank */
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* STM32H753VI: 2MB total flash = Bank 1 (1MB) + Bank 2 (1MB) */
  /* Bank 1: 0x08000000 - 0x080FFFFF */
  /* Bank 2: 0x08100000 - 0x081FFFFF */

  FLASH                             : ORIGIN = 0x08000000, LENGTH = 128K   /* Bootloader */
  BOOTLOADER_STATE                  : ORIGIN = 0x08020000, LENGTH = 128K   /* State */
  ACTIVE                            : ORIGIN = 0x08040000, LENGTH = 768K   /* Rest of Bank 1 for app */
  DFU                               : ORIGIN = 0x08100000, LENGTH = 1024K  /* Entire Bank 2 for DFU */
  RAM                         (rwx) : ORIGIN = 0x24000000, LENGTH = 512K
}

/* Bootloader symbols - state and active relative to FLASH origin */
__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(FLASH);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(FLASH);

__bootloader_active_start = ORIGIN(ACTIVE) - ORIGIN(FLASH);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - ORIGIN(FLASH);

/* DFU symbols - for dual-bank, relative to DFU origin (Bank 2) */
__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(DFU);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(DFU);
