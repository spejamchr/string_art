# frozen_string_literal: true

ROOT_DIR = File.join(__dir__, '..').freeze

CMD = File.join(ROOT_DIR, 'target', 'release', 'strings').freeze

EXPLORE_DIR = File.join(ROOT_DIR, 'explore').freeze

TEST_IMAGES = Dir.glob(File.join(ROOT_DIR, 'test_imgs', '*.png')).freeze

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
    nil => Time.now.to_i,
    r: opts[:pin_arrangement],
    m: opts[:max_strings],
    c: opts[:pin_count],
    s: opts[:step_size],
    a: opts[:string_alpha],
  }.map { |k, v| [k, v].compact.join('=') }.join('_') + '_' + File.basename(img, '.*')
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

vars = STORE.select { |_, v| v[:step_size] }
max_value = 2

rounds = (0...(max_value + 1)**vars.count)
  .map { |n| n.to_s(max_value + 1).rjust(vars.count, '0').split('').map(&:to_i) }
  .sort_by { |a| [a.max, a.sum] }

rounds.each do |step|
  opts = vars.transform_values.with_index { |v, i| v[:default] * v[:step_size]**step[i] }

  ARRANGEMENTS.each do |arrangement|
    opts[:pin_arrangement] = arrangement
    test_opts(opts)
  end
end
