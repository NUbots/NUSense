/* Application memory configuration for STM32H753VI */
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* Application starts at ACTIVE region in Bank 1 */
  FLASH                             : ORIGIN = 0x08000000, LENGTH = 1024K  /* Bank 1 */
  DFU                               : ORIGIN = 0x08100000, LENGTH = 1024K  /* Bank 2 */
  RAM                         (rwx) : ORIGIN = 0x24000000, LENGTH = 512K  /* 512 KiB of AXI ram */

  /* DTCM is directly linked to the CPU and very fast but can't be used with DMA */
  DTCM                        (rwx) : ORIGIN = 0x20000000, LENGTH = 128K  /* 128 KiB of DTCM */

  /* SRAM is slower than AXI but can be used with DMA */
  SRAM1                       (rwx) : ORIGIN = 0x30000000, LENGTH = 128K  /* 128 KiB of SRAM1 */
  SRAM2                       (rwx) : ORIGIN = 0x30020000, LENGTH = 128K  /* 128 KiB of SRAM2 */
  SRAM3                       (rwx) : ORIGIN = 0x30040000, LENGTH = 32K  /* 32 KiB of SRAM3 */
  SRAM4                       (rwx) : ORIGIN = 0x38000000, LENGTH = 64K  /* 64 KiB of SRAM4 */

  /* Backup SRAM is backed up by VBAT and can survive power cycles */
  BACKUP_SRAM               (rwx) : ORIGIN = 0x38800000, LENGTH = 4K    /* 4 KiB of Backup SRAM */
}
