/* See LICENSE of license details. */


#include <unistd.h>
#include "stub.h"

__attribute__((used)) int _isatty(int fd)
{
  if (fd == STDOUT_FILENO || fd == STDERR_FILENO)
    return 1;

  return 0;
}
