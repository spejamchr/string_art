# freeze_string_literals: true

require 'json'

file_name = ARGV[0]

json = JSON.parse(File.read(file_name))

pin_arrangement = json.dig("args", "pin_arrangement")

raise "Can only generate instructions for circular pin arrangements" if pin_arrangement != "Circle"

foreground_colors = json.dig("args", "foreground_colors") || []

raise "Can only generate instructions for single-color pieces" if foreground_colors.count != 1

PIN_COUNT = json.dig("args", "pin_count")

raise "Expected pin count to be an integer." unless PIN_COUNT.is_a?(Integer)
raise "Expected pin count to be positive" unless PIN_COUNT.positive?

pin_locations = json.fetch("pin_locations")

raise "Expected pin locations to be an array." unless pin_locations.is_a?(Array)

line_segments = json.fetch("line_segments")

raise "Expected line segments to be an array." unless line_segments.is_a?(Array)

numbered_line_segments = line_segments.map do |ls|
  [
    pin_locations.index(ls.fetch(0)) || raise("pin not found! #{ls[0]}"),
    pin_locations.index(ls.fetch(1)) || raise("pin not found! #{ls[1]}"),
  ].sort
end

def point_to_point_dist(point_a, point_b)
  [
    (point_a - point_b) % PIN_COUNT,
    (point_b - point_a) % PIN_COUNT,
  ].min
end

def point_to_segment_dists(point_a, segment)
  segment.map { |point_b| point_to_point_dist(point_a, point_b) }
end

def find_closest_but_not(point, segment, all_segments)
  all_segments.reject { |s| s == segment }.min_by do |other_segment|
    dists = point_to_segment_dists(point, other_segment)
    return other_segment if dists.any?(0)

    dists.min
  end
end

ordered_segments = []
ordered_segments << numbered_line_segments.shift

while numbered_line_segments.any?
  current_point = ordered_segments.last.last
  next_segment = find_closest_but_not(current_point, ordered_segments.last.sort, numbered_line_segments)
  ordered_segments << next_segment.sort_by { |point_b| point_to_point_dist(current_point, point_b) }
  numbered_line_segments.delete_at(numbered_line_segments.index(next_segment))
end

TOTAL_SEGS = ordered_segments.count
DIGITS = Math.log(TOTAL_SEGS, 26).ceil
BASE = (TOTAL_SEGS**(1r / DIGITS)).ceil

REGULAR_BASE = (0..BASE - 1).map { |n| n.to_s(BASE) }.freeze
LETTER_BASE = %w[
  Alfa
  Bravo
  Charlie
  Delta
  Echo
  Foxtrot
  Golf
  Hotel
  India
  Juliett
  Kilo
  Lima
  Mike
  November
  Oscar
  Papa
  Quebec
  Romeo
  Sierra
  Tango
  Uniform
  Victor
  Whiskey
  Xray
  Yankee
  Zulu
].first(BASE).freeze
LOOKUP = REGULAR_BASE.zip(LETTER_BASE).to_h

def num_to_letters(num)
  num.to_s(BASE).split('').map { |a| LOOKUP[a] }
end

def num_to_padded_letters(num)
  a = num_to_letters(num)
  ([LETTER_BASE.first] * (DIGITS - a.count) + a).join(' ')
end

total = ordered_segments.count
last_point = ordered_segments.first.first
ordered_segments.each_with_index do |s, i|
  puts if (i % BASE).zero?
  puts "You have strung #{i} strings (there are #{total - i} left).\n" if (i % BASE**2).zero?
  puts "(Around from pin '#{last_point}' to pin '#{s[0]}')" if last_point != s[0]
  puts "[#{num_to_padded_letters(i)}] From pin '#{s[0]}' to pin '#{s[1]}'."
  last_point = s[1]
end
