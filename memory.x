/* Application memory configuration for STM32H753VI */
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* Application starts at ACTIVE region in Bank 1 */
  FLASH                             : ORIGIN = 0x08000000, LENGTH = 1024K  /* Bank 1 */
  DFU                               : ORIGIN = 0x08100000, LENGTH = 1024K  /* Bank 2 */
  RAM                         (rwx) : ORIGIN = 0x24000000, LENGTH = 512K
}
