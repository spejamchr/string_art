# frozen_string_literal: true

require 'json'
require 'csv'

EXPLORE_DIR = File.join(__dir__, '..', 'explore').freeze

JSONS = Dir.glob(File.join(EXPLORE_DIR, '*.json')).freeze

CSV_PATH = File.join(EXPLORE_DIR, 'data.csv').freeze
MD_PATH = File.join(EXPLORE_DIR, 'data.md').freeze

HEADERS = %w[
  max_strings
  used_strings
  pin_count
  used_pins
  step_size
  string_alpha
  pin_arrangement
  image_height
  image_width
  initial_score
  final_score
  elapsed_seconds
  score_change
  score_change_per_second
  path
].freeze

def flat_json_hash(path, hash)
  score_change = hash['final_score'] - hash['initial_score']
  hash
    .merge(hash['args'])
    .merge(
      'path' => path,
      'used_strings' => hash['line_segments'].count,
      'used_pins' => hash['pin_locations'].count,
      'score_change' => score_change,
      'score_change_per_second' => score_change / hash['elapsed_seconds'],
    )
    .slice(*HEADERS)
end

CSV.open(CSV_PATH, 'wb') do |csv|
  csv << HEADERS

  JSONS
    .map { |path| [path, File.read(path)] }
    .map { |path, json| [path, JSON.parse(json)] }
    .map { |path, hash| flat_json_hash(path, hash) }
    .sort_by { |hash| hash['score_change_per_second'] }
    .map { |hash| HEADERS.map { |header| hash[header] } }
    .each { |array| csv << array }
end
