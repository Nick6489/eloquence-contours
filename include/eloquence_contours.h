#ifndef ELOQUENCE_CONTOURS_H
#define ELOQUENCE_CONTOURS_H
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif
/* cdecl; resolve exports with GetProcAddress or link a generated import library.
 * All PCM is mono signed int16. Counts/capacities are samples, not bytes.
 * One live handle per stream, externally serialized. No overlapping buffers.
 * Destroy before unloading the library. NULL destroy is safe.
 * Rates: 11025 or 16000. Modes: 0 Raw, 1 legacy/default (Classic Presence,
 * native Smooth), 2 Presence, 3 Presence treble (11025 Hz only).
 * Classic modes 1/2/3 output 22050 Hz; otherwise
 * output rate equals input rate. Unsupported creation returns NULL.
 */
typedef struct Contour ec_contour;
uint32_t ec_contour_abi_version(void);
ec_contour *ec_contour_create(uint32_t rate, uint32_t mode);
void ec_contour_destroy(ec_contour *handle);
int32_t ec_contour_reset(ec_contour *handle);
uint32_t ec_contour_output_rate(const ec_contour *handle);
/* SIZE_MAX on invalid count/handle. Does not change state. */
size_t ec_contour_output_count(const ec_contour *handle, size_t count);
/* Returns 0, -1 invalid argument, -2 capacity too small; errors preserve state.
 * Zero samples permit NULL buffers. Writes exactly output_count samples.
 */
int32_t ec_contour_process(ec_contour *handle, const int16_t *input,
    size_t count, int16_t *output, size_t capacity);
#ifdef __cplusplus
}
#endif
#endif
