namespace :build do
  desc 'Rust'
  task :rust do
    Shell.sh 'rustup install stable'
    Shell.sh 'rustup default stable'
  end

  desc 'Build extractor'
  task :extractor do
    Shell.chdir(Paths::EXTRACTOR) do
      Shell.sh 'cargo build --release'
      Reporter.add(Jobs::Building, Owner::Extractor, 'built', '')
    end
  end

  desc 'Build lib'
  task :lib do
    Shell.sh 'cargo build --release'
    Reporter.add(Jobs::Building, Owner::Lib, 'built', '')
  end

  desc 'build'
  task envvars: ['build:rust', 'build:extractor', 'build:lib'] do
    Reporter.print
  end
end

namespace :test do
  desc 'Build extractor'
  task :extractor do
    Shell.chdir(Paths::EXTRACTOR) do
      Shell.sh 'cargo test'
      Reporter.add(Jobs::Test, Owner::Extractor, 'tested', '')
    end
  end

  desc 'Build lib'
  task :lib do
    Shell.sh 'cargo test'
    Reporter.add(Jobs::Test, Owner::Lib, 'tested', '')
  end

  desc 'test'
  task envvars: ['build:envvars', 'test:extractor', 'test:lib'] do
    Reporter.print
  end
end

namespace :clippy do
  desc 'Clippy update to nightly'
  task :nightly do
    Shell.sh 'rustup install nightly'
    Shell.sh 'rustup default nightly'
    Shell.sh 'rustup component add --toolchain=nightly clippy-preview'
  end

  desc 'Clippy extractor'
  task :extractor do
    Reporter.add(Jobs::Clippy, Owner::Extractor, 'checked', '')
    Shell.chdir(Paths::EXTRACTOR) do
      Shell.sh Paths::CLIPPY_NIGHTLY
    end
  end

  desc 'Clippy lib'
  task :lib do
    Rake::Task['build:extractor'].invoke
    Reporter.add(Jobs::Clippy, Owner::Lib, 'checked', '')
    Shell.sh Paths::CLIPPY_NIGHTLY
  end

  desc 'Clippy all'
  task envvars: ['clippy:nightly', 'clippy:extractor', 'clippy:lib'] do
    Reporter.print
  end
end

namespace :clean do
  desc 'Clean extractor'
  task :extractor do
    Shell.rm_rf("#{Paths::EXTRACTOR}/target")
    Reporter.add(Jobs::Clearing, Owner::Extractor, "removed: #{Paths::EXTRACTOR}/target", '')
  end

  desc 'Clean lib'
  task :lib do
    Shell.rm_rf('./target')
    Reporter.add(Jobs::Clearing, Owner::Lib, 'removed: ./target', '')
  end

  desc 'Clean all'
  task envvars: ['clean:extractor', 'clean:lib'] do
    Reporter.print
  end
end

task :default do
  Rake::Task['clippy:envvars'].invoke
  Rake::Task['test:envvars'].invoke
  Rake::Task['build:envvars'].invoke
end
