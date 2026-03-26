FACTOR = 100

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
        new_zr = (zr * zr - zi * zi) / FACTOR + ar
        new_zi = (2 * zr * zi)        / FACTOR + bi
        zr = new_zr
        zi = new_zi
    end
    zr * zr + zi * zi < 4 * FACTOR * FACTOR
end

100.step(to: -100, by: -5) do |y|
    (-200).step(to: 50, by: 3) do |x|
        print mandelbrot(x, y) ? '*' : ' '
    end
    puts
end

puts mandelbrot(0, 0)
puts mandelbrot(150, 30)