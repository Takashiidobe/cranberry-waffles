raise "zero must be truthy" unless (0 ? true : false)
raise "empty strings must be truthy" unless ("" ? true : false)
raise "negative division must floor" unless -7 / 3 == -3
raise "modulo follows the divisor" unless -7 % 3 == 2
raise "integers must grow beyond 64 bits" unless (2**63 + 1) - 2**63 == 1

puts "semantics OK"
