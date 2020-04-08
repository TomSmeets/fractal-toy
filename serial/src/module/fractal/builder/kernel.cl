__kernel void add(write_only image2d_t image, float max_iter, double offset_x, double offset_y, double zoom) {
    int2 coord = (int2)(get_global_id(0), get_global_id(1));
    double2 image_pos = (double2)((double) coord.x / 128.0, (double) coord.y / 128.0);

    double2 c = ((double2)((double) image_pos.x, (double) image_pos.y));

    // screen coords 0 - 1
    c.y = 1.0 - c.y;

    // -1 , 1
    c = zoom * c + (double2)(offset_x, offset_y);

    double2 z = (double2)(0.0, 0.0);

    float n = 0.0f;
    double2 tmp;
    while (n < max_iter) {

        @ALGORITHM@

        if (z.x*z.x + z.y*z.y > 64.0*64.0) {
            break;
        }

        n += @INC@;
    }

    n += - log2(log2(z.x*z.x+z.y*z.y)) + 4.0;

    // convert to hsv
    float hue = n / 64.0;
    float v   = n / max_iter;
    v *= v;
    v = 1.0f - v;

    float sat = v;
    float val = v;

    hue = fmod(hue, 1.0f)*6.0f;
    int part = (int) hue;
    float fract = fmod(hue, 1.0f);

    // upper limit
    float max = val;
    // lower limit
    float min = val - val * sat;
    // increasing slope
    float inc = fract * max + (1.0f - fract) * min;
    // decreasing slope
    float dec = fract * min + (1.0f - fract) * max;

    float4 pixel = (float4)(1.0, 1.0, 1.0, 1.0);

    switch(part) {
        case 0: pixel = (float4)(1.0f, max, inc, min); break;
        case 1: pixel = (float4)(1.0f, dec, max, min); break;
        case 2: pixel = (float4)(1.0f, min, max, inc); break;
        case 3: pixel = (float4)(1.0f, min, dec, max); break;
        case 4: pixel = (float4)(1.0f, inc, min, max); break;
        case 5: pixel = (float4)(1.0f, max, min, dec); break;
        default: pixel = (float4)(1.0f, max, max, max); break;
    }

    // write pixel to image
    write_imagef(image, coord, pixel);
}
