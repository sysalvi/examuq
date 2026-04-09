<?php

namespace App\Http\Controllers\Web;

use App\Http\Controllers\Controller;
use App\Models\Exam;
use App\Models\ExamSession;
use App\Models\LaunchToken;
use Carbon\Carbon;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Str;
use Illuminate\View\View;

class StudentLoginController extends Controller
{
    public function show(): View
    {
        return view('exam.student-login');
    }

    public function submit(Request $request): RedirectResponse
    {
        $payload = $request->validate([
            'display_name' => ['required', 'string', 'max:255'],
            'class_room' => ['required', 'string', 'max:255'],
            'token_global' => ['required', 'string', 'max:255'],
        ]);

        $clientType = (string) $request->attributes->get('examuqClientType');
        $deviceId = $request->attributes->get('examuqDeviceId');

        $now = Carbon::now();

        $exam = Exam::query()
            ->where('token_global', $payload['token_global'])
            ->where('is_active', true)
            ->orderByDesc('id')
            ->first();

        if (! $exam) {
            return back()
                ->withInput($request->except('token_global'))
                ->withErrors([
                    'token_global' => 'Token tidak valid atau ujian belum diaktifkan.',
                ]);
        }

        $session = ExamSession::query()->create([
            'exam_id' => $exam->id,
            'display_name' => $payload['display_name'],
            'class_room' => $payload['class_room'],
            'client_type' => $clientType,
            'device_id' => is_string($deviceId) ? $deviceId : null,
            'status' => 'active',
            'started_at' => $now,
            'last_heartbeat_at' => $now,
            'ip_address' => $request->ip(),
            'user_agent' => $request->userAgent(),
        ]);

        $rawToken = Str::random(64);
        $expiresAt = $now->copy()->addMinutes(5);

        LaunchToken::query()->create([
            'exam_id' => $exam->id,
            'exam_session_id' => $session->id,
            'token_hash' => hash('sha256', $rawToken),
            'issued_for_client' => $clientType,
            'issued_for_ip' => $request->ip(),
            'payload_json' => [
                'display_name' => $payload['display_name'],
                'class_room' => $payload['class_room'],
            ],
            'expires_at' => $expiresAt,
        ]);

        return redirect()->route('exam.confirm', [
            'lt' => $rawToken,
        ]);
    }
}
