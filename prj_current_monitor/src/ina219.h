#ifndef INA219_H
#define INA219_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

#define INA219_ADDR 0x40

// Raw values for USB streaming
typedef struct {
    uint16_t voltage_mv;
    int16_t current_ma;
    uint16_t power_mw;
} ina219_data_t;

bool ina219_init(void);
bool ina219_read_all(ina219_data_t *data);

#ifdef __cplusplus
}
#endif

#endif // INA219_H
