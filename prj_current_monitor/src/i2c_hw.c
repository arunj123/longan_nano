#include "i2c_hw.h"

#define I2C0_SPEED 100000

void i2c_hw_init(void) {
    /* enable GPIOB clock */
    rcu_periph_clock_enable(RCU_GPIOB);
    /* enable I2C0 clock */
    rcu_periph_clock_enable(RCU_I2C0);

    /* connect PB6 to I2C0_SCL */
    /* connect PB7 to I2C0_SDA */
    gpio_init(GPIOB, GPIO_MODE_AF_OD, GPIO_OSPEED_50MHZ, GPIO_PIN_6 | GPIO_PIN_7);

    /* configure I2C0 clock */
    i2c_clock_config(I2C0, I2C0_SPEED, I2C_DTCY_2);
    /* configure I2C0 address */
    i2c_mode_addr_config(I2C0, I2C_I2CMODE_ENABLE, I2C_ADDFORMAT_7BITS, 0x00);
    /* enable I2C0 */
    i2c_enable(I2C0);
    /* enable acknowledge */
    i2c_ack_config(I2C0, I2C_ACK_ENABLE);
}

bool i2c_hw_write_reg(uint8_t dev_addr, uint8_t reg_addr, uint16_t value) {
    /* wait until I2C bus is idle */
    while(i2c_flag_get(I2C0, I2C_FLAG_I2CBSY));

    /* send a start condition to I2C bus */
    i2c_start_on_bus(I2C0);
    /* wait until SBSEND bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_SBSEND));

    /* send slave address to I2C bus */
    i2c_master_addressing(I2C0, (uint8_t)(dev_addr << 1), I2C_TRANSMITTER);
    /* wait until ADDSEND bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_ADDSEND));
    /* clear ADDSEND bit */
    i2c_flag_clear(I2C0, I2C_FLAG_ADDSEND);
    /* wait until TBE bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_TBE));

    /* send register address */
    i2c_data_transmit(I2C0, reg_addr);
    /* wait until TBE bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_TBE));

    /* send data MSB */
    i2c_data_transmit(I2C0, (uint8_t)(value >> 8));
    /* wait until TBE bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_TBE));

    /* send data LSB */
    i2c_data_transmit(I2C0, (uint8_t)(value & 0xFF));
    /* wait until TBE bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_TBE));

    /* send a stop condition to I2C bus */
    i2c_stop_on_bus(I2C0);
    /* wait until STOP bit is cleared */
    while(I2C_CTL0(I2C0) & I2C_CTL0_STOP);

    return TRUE;
}

bool i2c_hw_read_reg(uint8_t dev_addr, uint8_t reg_addr, uint16_t *value) {
    /* wait until I2C bus is idle */
    while(i2c_flag_get(I2C0, I2C_FLAG_I2CBSY));

    /* send a start condition to I2C bus */
    i2c_start_on_bus(I2C0);
    /* wait until SBSEND bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_SBSEND));

    /* send slave address to I2C bus */
    i2c_master_addressing(I2C0, (uint8_t)(dev_addr << 1), I2C_TRANSMITTER);
    /* wait until ADDSEND bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_ADDSEND));
    /* clear ADDSEND bit */
    i2c_flag_clear(I2C0, I2C_FLAG_ADDSEND);
    /* wait until TBE bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_TBE));

    /* send register address */
    i2c_data_transmit(I2C0, reg_addr);
    /* wait until TBE bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_TBE));

    /* send a start condition to I2C bus */
    i2c_start_on_bus(I2C0);
    /* wait until SBSEND bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_SBSEND));

    /* send slave address to I2C bus */
    i2c_master_addressing(I2C0, (uint8_t)((dev_addr << 1) | 1), I2C_RECEIVER);
    /* wait until ADDSEND bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_ADDSEND));
    /* clear ADDSEND bit */
    i2c_flag_clear(I2C0, I2C_FLAG_ADDSEND);

    /* wait until RBNE bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_RBNE));
    /* read data MSB */
    uint8_t msb = i2c_data_receive(I2C0);

    /* disable acknowledge before the last byte */
    i2c_ack_config(I2C0, I2C_ACK_DISABLE);

    /* send a stop condition to I2C bus */
    i2c_stop_on_bus(I2C0);

    /* wait until RBNE bit is set */
    while(!i2c_flag_get(I2C0, I2C_FLAG_RBNE));
    /* read data LSB */
    uint8_t lsb = i2c_data_receive(I2C0);

    /* wait until STOP bit is cleared */
    while(I2C_CTL0(I2C0) & I2C_CTL0_STOP);

    /* enable acknowledge for next transfers */
    i2c_ack_config(I2C0, I2C_ACK_ENABLE);

    *value = (uint16_t)((msb << 8) | lsb);
    return TRUE;
}
