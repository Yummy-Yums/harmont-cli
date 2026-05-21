<?php
namespace Tests\Unit;

use App\Models\Example;
use PHPUnit\Framework\TestCase;

class ExampleTest extends TestCase {
    public function test_adds(): void {
        $this->assertSame(5, Example::add(2, 3));
    }
}
