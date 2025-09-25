#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/shm.h>
#include <sys/stat.h>
#include <unistd.h>

/* 64bit arch MACRO */
#if (defined(__x86_64__) || defined(__arm64__) || defined(__aarch64__))
#define WORD_SIZE_64 1
#endif

#if __GNUC__ < 6
#ifndef likely
#define likely(_x) (_x)
#endif
#ifndef unlikely
#define unlikely(_x) (_x)
#endif
#else
#ifndef likely
#define likely(_x) __builtin_expect(!!(_x), 1)
#endif
#ifndef unlikely
#define unlikely(_x) __builtin_expect(!!(_x), 0)
#endif
#endif

int destroy_shmem(int id)
{
  if (-1 == shmctl(id, IPC_RMID, NULL))
  {
    return -1;
  }
  return 0;
}

/* Check if the current execution path brings anything new to the table.
   Update virgin bits to reflect the finds. Returns 1 if the only change is
   the hit-count for a particular tuple; 2 if there are new tuples seen.
   Updates the map, so subsequent calls will always return 0.

   This function is called after every exec() on a fairly large buffer, so
   it needs to be fast. We do this in 32-bit and 64-bit flavors. */

uint8_t has_new_bits(uint8_t *virgin_map, uint8_t *shm_map, uint32_t map_size)
{

#ifdef WORD_SIZE_64

  uint64_t *current = (uint64_t *)shm_map;
  uint64_t *virgin = (uint64_t *)virgin_map;

  uint32_t i = (map_size >> 3);

#else

  uint32_t *current = (uint32_t *)shm_map;
  uint32_t *virgin = (uint32_t *)virgin_map;

  uint32_t i = (map_size >> 2);

#endif /* ^WORD_SIZE_64 */

  uint8_t ret = 0;

  while (i--)
  {

    /* Optimize for (*current & *virgin) == 0 - i.e., no bits in current bitmap
       that have not been already cleared from the virgin map - since this will
       almost always be the case. */

    if (unlikely(*current) && unlikely(*current & *virgin))
    {

      if (likely(ret < 2))
      {

        uint8_t *cur = (uint8_t *)current;
        uint8_t *vir = (uint8_t *)virgin;

        /* Looks like we have not found any new bytes yet; see if any non-zero
           bytes in current[] are pristine in virgin[]. */

#ifdef WORD_SIZE_64

        if ((cur[0] && vir[0] == 0xff) || (cur[1] && vir[1] == 0xff) ||
            (cur[2] && vir[2] == 0xff) || (cur[3] && vir[3] == 0xff) ||
            (cur[4] && vir[4] == 0xff) || (cur[5] && vir[5] == 0xff) ||
            (cur[6] && vir[6] == 0xff) || (cur[7] && vir[7] == 0xff))
          ret = 2;
        else
          ret = 1;

#else

        if ((cur[0] && vir[0] == 0xff) || (cur[1] && vir[1] == 0xff) ||
            (cur[2] && vir[2] == 0xff) || (cur[3] && vir[3] == 0xff))
          ret = 2;
        else
          ret = 1;

#endif /* ^WORD_SIZE_64 */
      }

      *virgin &= ~*current;
    }

    current++;
    virgin++;
  }

  return ret;
}
