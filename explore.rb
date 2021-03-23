# frozen_string_literal: true

CMD = File.join(__dir__, 'target', 'release', 'strings').freeze

EXPLORE_DIR = File.join(__dir__, 'explore').freeze

TEST_IMAGES = Dir.glob(File.join(__dir__, 'test_imgs', '*.png')).freeze

def make_store(cmd, default, step_size)
  {
    cmd: cmd,
    default: default,
    step_size: step_size,
  }.freeze
end

ARRANGEMENTS = %w[
  perimeter
  grid
  circle
  random
].freeze

STORE = {
  output_filepath: make_store('--output-filepath', nil, nil),
  data_filepath: make_store('--data-filepath', nil, nil),
  pin_arrangement: make_store('--pin-arrangement', nil, nil),
  max_strings: make_store('--max-strings', 300, 1.15),
  pin_count: make_store('--pin-count', 80, 1.15),
  step_size: make_store('--step-size', 1.0, 0.88),
  string_alpha: make_store('--string-alpha', 1.0, 0.88),
}.freeze

class Monotonic
  def initialize
    @moment = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  end

  def elapsed_seconds
    Process.clock_gettime(Process::CLOCK_MONOTONIC) - @moment
  end
end

def run_cmd(filepath, opts)
  cmd_str = opts
    .slice(*STORE.keys)
    .transform_keys { |k| STORE[k][:cmd] }
    .map { |k, v| "#{k}=#{v}" }
    .join(' ')
    .yield_self { |options| "#{CMD} #{filepath} #{options}" }
  started_at = Monotonic.new
  puts cmd_str
  system(cmd_str)
  [started_at.elapsed_seconds, cmd_str]
end

def output_filename(img, opts)
  {
    now: Time.now.to_i,
    r: opts[:pin_arrangement],
    m: opts[:max_strings],
    c: opts[:pin_count],
    s: opts[:step_size],
    a: opts[:string_alpha],
  }.map { |k, v| "#{k}=#{v}" }.join('_') + '_' + File.basename(img, '.*')
end

def id_filepath(img, opts, ext)
  "'#{File.join(EXPLORE_DIR, output_filename(img, opts))}#{ext}'"
end

def test_opts(opts)
  opts[:max_strings] = opts[:max_strings].to_i
  opts[:pin_count] = opts[:pin_count].to_i
  TEST_IMAGES.each do |img|
    opts[:output_filepath] = id_filepath(img, opts, '.png')
    opts[:data_filepath] = id_filepath(img, opts, '.json')
    elapsed_seconds, cmd = run_cmd(img, opts)
    puts "[#{elapsed_seconds}] #{cmd}"
  end
end

(1..100).each do |step|
  STORE.values.select { |v| v[:step_size] }.map { |v| v[:cmd] }.each do |target_cmd|
    opts = STORE.transform_values do |v|
      v[:cmd] == target_cmd ? v[:default] * v[:step_size]**step : v[:default]
    end
    ARRANGEMENTS.each do |arrangement|
      opts[:pin_arrangement] = arrangement
      test_opts(opts)
    end
  end

  opts = STORE.transform_values do |v|
    v[:step_size] ? v[:default] * v[:step_size]**step : v[:default]
  end
  ARRANGEMENTS.each do |arrangement|
    opts[:pin_arrangement] = arrangement
    test_opts(opts)
  end
end
