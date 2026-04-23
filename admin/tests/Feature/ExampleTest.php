<?php

namespace Tests\Feature;

use App\Models\Exam;
use App\Models\ExamSession;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\URL;
use Tests\TestCase;

class ExampleTest extends TestCase
{
    use RefreshDatabase;

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

    public function test_student_login_submit_reuses_backend_client_identity_from_session(): void
    {
        $user = User::factory()->create();

        $exam = Exam::query()->create([
            'title' => 'Ujian Backend Flow',
            'subject' => 'Teknologi',
            'class_room' => '12-A',
            'exam_url' => 'https://example.com/exam',
            'duration_minutes' => 90,
            'is_active' => true,
            'created_by' => $user->id,
        ]);

        $response = $this->get('/?source=client&client_type=desktop_client&device_id=device-123')
            ->assertOk();

        preg_match('/<form method="post" action="([^"]+)">/', $response->getContent(), $matches);
        $submitUrl = html_entity_decode($matches[1] ?? '', ENT_QUOTES);

        $this->assertNotSame('', $submitUrl);

        $response = $this->post($submitUrl, [
            'display_name' => 'Dokun',
            'class_room' => '12-A',
            'token_global' => $exam->token_global,
        ]);

        $response->assertRedirect();
        $this->assertStringContainsString('/exam-confirm?lt=', $response->headers->get('Location', ''));

        $session = ExamSession::query()->firstOrFail();

        $this->assertSame('desktop_client', $session->client_type);
        $this->assertSame('device-123', $session->device_id);
    }

    public function test_student_login_submit_requires_signed_submit_url(): void
    {
        $user = User::factory()->create();

        $exam = Exam::query()->create([
            'title' => 'Ujian Signed Submit',
            'subject' => 'Teknologi',
            'class_room' => '12-A',
            'exam_url' => 'https://example.com/exam',
            'duration_minutes' => 90,
            'is_active' => true,
            'created_by' => $user->id,
        ]);

        $unsigned = '/student/login?source=client&client_type=desktop_client&device_id=device-123';

        $this->post($unsigned, [
            'display_name' => 'Dokun',
            'class_room' => '12-A',
            'token_global' => $exam->token_global,
        ])->assertForbidden();
    }

    public function test_student_login_page_generates_temporary_signed_submit_url(): void
    {
        $response = $this->get('/?source=client&client_type=desktop_client&device_id=device-123')
            ->assertOk();

        preg_match('/<form method="post" action="([^"]+)">/', $response->getContent(), $matches);
        $submitUrl = html_entity_decode($matches[1] ?? '', ENT_QUOTES);

        $this->assertNotSame('', $submitUrl);
        $this->assertTrue(URL::hasValidSignature(request()->create($submitUrl, 'POST')));
        $this->assertStringContainsString('client_type=desktop_client', $submitUrl);
        $this->assertStringContainsString('device_id=device-123', $submitUrl);
    }

    public function test_exam_gateway_redirects_directly_with_client_identity(): void
    {
        $user = User::factory()->create();

        $exam = Exam::query()->create([
            'title' => 'Ujian Player Client Identity',
            'subject' => 'Teknologi',
            'class_room' => '12-A',
            'exam_url' => 'http://10.10.20.2:6997/?foo=bar',
            'duration_minutes' => 90,
            'is_active' => true,
            'created_by' => $user->id,
        ]);

        $loginPage = $this->get('/?source=client&client_type=desktop_client&device_id=device-123')
            ->assertOk();

        preg_match('/<form method="post" action="([^"]+)">/', $loginPage->getContent(), $matches);
        $submitUrl = html_entity_decode($matches[1] ?? '', ENT_QUOTES);

        $loginSubmit = $this->post($submitUrl, [
            'display_name' => 'Dokun',
            'class_room' => '12-A',
            'token_global' => $exam->token_global,
        ])->assertRedirect();

        $confirmUrl = $loginSubmit->headers->get('Location', '');
        $this->assertStringContainsString('/exam-confirm?lt=', $confirmUrl);

        $confirmPage = $this->get($confirmUrl)->assertOk();
        preg_match('/<input type="hidden" name="lt" value="([^"]+)">/', $confirmPage->getContent(), $confirmMatch);
        $launchToken = $confirmMatch[1] ?? '';

        $this->assertNotSame('', $launchToken);

        $gatewayResponse = $this->get('/exam-gateway?lt='.$launchToken);
        $gatewayResponse->assertRedirect();

        $target = $gatewayResponse->headers->get('Location', '');
        $this->assertStringContainsString('http://10.10.20.2:6997/?foo=bar&source=client&client_type=desktop_client&device_id=device-123', $target);

        $query = parse_url($target, PHP_URL_QUERY);
        $this->assertIsString($query);

        parse_str($query, $overlay);
        $this->assertSame('Dokun', $overlay['examuq_display_name'] ?? null);
        $this->assertSame('https://return.examuq.invalid/launcher', $overlay['examuq_return_url'] ?? null);
        $this->assertNotEmpty($overlay['examuq_session_id'] ?? '');
        $this->assertNotEmpty($overlay['examuq_deadline_at'] ?? '');
        $this->assertSame('http://localhost', $overlay['examuq_api_base'] ?? null);
    }

    public function test_exam_gateway_redirects_directly_for_extension_client(): void
    {
        $user = User::factory()->create();

        $exam = Exam::query()->create([
            'title' => 'Ujian Extension Fullscreen Guard',
            'subject' => 'Teknologi',
            'class_room' => '12-A',
            'exam_url' => 'http://10.10.20.2:6997/',
            'duration_minutes' => 90,
            'is_active' => true,
            'created_by' => $user->id,
        ]);

        $loginPage = $this->get('/?source=extension&client_type=chrome_extension&device_id=ext-1')
            ->assertOk();

        preg_match('/<form method="post" action="([^"]+)">/', $loginPage->getContent(), $matches);
        $submitUrl = html_entity_decode($matches[1] ?? '', ENT_QUOTES);

        $loginSubmit = $this->post($submitUrl, [
            'display_name' => 'Dokun',
            'class_room' => '12-A',
            'token_global' => $exam->token_global,
        ])->assertRedirect();

        $confirmUrl = $loginSubmit->headers->get('Location', '');
        $confirmPage = $this->get($confirmUrl)->assertOk();
        preg_match('/<input type="hidden" name="lt" value="([^"]+)">/', $confirmPage->getContent(), $confirmMatch);
        $launchToken = $confirmMatch[1] ?? '';

        $gatewayResponse = $this->get('/exam-gateway?lt='.$launchToken);
        $gatewayResponse->assertRedirect();

        $target = $gatewayResponse->headers->get('Location', '');
        $this->assertStringContainsString('http://10.10.20.2:6997/?source=extension&client_type=chrome_extension&device_id=ext-1', $target);

        $query = parse_url($target, PHP_URL_QUERY);
        $this->assertIsString($query);

        parse_str($query, $overlay);
        $this->assertSame('Dokun', $overlay['examuq_display_name'] ?? null);
        $this->assertSame('https://return.examuq.invalid/launcher', $overlay['examuq_return_url'] ?? null);
        $this->assertNotEmpty($overlay['examuq_session_id'] ?? '');
        $this->assertNotEmpty($overlay['examuq_deadline_at'] ?? '');
        $this->assertSame('http://localhost', $overlay['examuq_api_base'] ?? null);
    }
}
