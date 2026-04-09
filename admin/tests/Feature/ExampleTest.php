<?php

namespace Tests\Feature;

use Tests\TestCase;

class ExampleTest extends TestCase
{
    public function test_student_login_page_requires_examuq_client_header(): void
    {
        $response = $this->get('/');

        $response->assertStatus(403);
    }

    public function test_student_login_page_is_accessible_with_examuq_client_header(): void
    {
        $response = $this->withHeaders([
            'X-ExamUQ-Client-Type' => 'desktop_client',
        ])->get('/');

        $response->assertStatus(200);
        $response->assertSee('Login Siswa');
    }
}
