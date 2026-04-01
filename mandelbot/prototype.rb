FACTOR = 50

# Complex numbers represented as two integers (real, imag) scaled by FACTOR.
# e.g., 1.5 + 0.3i => (150, 30)
#
# Ranges in fixed-point:
#   y: 1.0..−1.0 step −0.05  →  100..−100 step −5
#   x: −2.0..0.5 step ~0.032 → −200..50   step 3

def mandelbrot(ar, bi)
    zr = 0
    zi = 0
    4.times do
        # z = z*z + a  (fixed-point: divide by FACTOR after multiply)
        if (zr*zr) >= 2147483647
            puts "Overflow: #{zr} * #{zr} = #{zr*zr}, #{ar} + #{bi}i"
        end
        new_zr = (zr * zr - zi * zi) / FACTOR + ar
        new_zi = (2 * zr * zi)        / FACTOR + bi
        zr = new_zr
        zi = new_zi
    end
    zr * zr + zi * zi
end

70.step(to: -70, by: -5) do |y|
    (-100).step(to: 50, by: 3) do |x|
        v = mandelbrot(x, y)
        if v < 2 * FACTOR * FACTOR
            print '#'
        elsif v < 4 * FACTOR * FACTOR
            print '*'
        elsif v < 5 * FACTOR * FACTOR
            print '+'
        elsif v < 6 * FACTOR * FACTOR
            print '-'
        else
            print ' '
        end
    end
    puts
end
