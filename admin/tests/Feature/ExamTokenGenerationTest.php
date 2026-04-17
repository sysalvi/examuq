<?php

namespace Tests\Feature;

use App\Models\Exam;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class ExamTokenGenerationTest extends TestCase
{
    use RefreshDatabase;

    public function test_exam_creation_generates_six_character_alphanumeric_token(): void
    {
        $user = User::factory()->create();

        $exam = Exam::create([
            'title' => 'Ujian Matematika',
            'subject' => 'Matematika',
            'class_room' => '12-A',
            'exam_url' => 'https://example.com/exams/math',
            'duration_minutes' => 90,
            'created_by' => $user->id,
        ]);

        $this->assertMatchesRegularExpression('/^[A-Z0-9]{6}$/', $exam->token_global);
    }

    public function test_generated_tokens_are_unique_for_multiple_exams(): void
    {
        $user = User::factory()->create();
        $tokens = [];

        for ($index = 1; $index <= 10; $index++) {
            $tokens[] = Exam::create([
                'title' => "Ujian {$index}",
                'subject' => 'Matematika',
                'class_room' => '12-A',
                'exam_url' => "https://example.com/exams/{$index}",
                'duration_minutes' => 90,
                'created_by' => $user->id,
            ])->token_global;
        }

        $this->assertCount(10, array_unique($tokens));
    }
}
