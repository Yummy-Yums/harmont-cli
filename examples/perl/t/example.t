use strict;
use warnings;
use Test::More tests => 1;
use lib 'lib';
use Example;

is(Example::add(2, 3), 5, 'adds');
