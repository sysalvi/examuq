<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Models\Exam;
use App\Models\ExamSession;
use App\Models\LaunchToken;
use Carbon\Carbon;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Str;

class LaunchController extends Controller
{
    public function requestLaunch(Request $request): JsonResponse
    {
        $payload = $request->validate([
            'exam_id' => ['required', 'integer', 'exists:exams,id'],
            'display_name' => ['required', 'string', 'max:255'],
            'class_room' => ['required', 'string', 'max:255'],
            'token_global' => ['required', 'string', 'max:255'],
            'client_type' => ['required', 'string', 'in:desktop_client,chrome_extension'],
            'device_id' => ['nullable', 'string', 'max:255'],
        ]);

        $exam = Exam::query()->findOrFail($payload['exam_id']);

        if (! $exam->is_active || $exam->token_global !== $payload['token_global']) {
            return response()->json([
                'message' => 'Token tidak valid atau ujian tidak aktif.',
            ], 422);
        }

        $now = Carbon::now();

        $session = ExamSession::query()->create([
            'exam_id' => $exam->id,
            'display_name' => $payload['display_name'],
            'class_room' => $payload['class_room'],
            'client_type' => $payload['client_type'],
            'device_id' => $payload['device_id'] ?? null,
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
            'issued_for_client' => $payload['client_type'],
            'issued_for_ip' => $request->ip(),
            'payload_json' => [
                'display_name' => $payload['display_name'],
                'class_room' => $payload['class_room'],
            ],
            'expires_at' => $expiresAt,
        ]);

        return response()->json([
            'message' => 'Launch token berhasil dibuat.',
            'session_id' => $session->id,
            'launch_token' => $rawToken,
            'expires_at' => $expiresAt->toIso8601String(),
            'redirect_url' => url('/exam-gateway?lt='.$rawToken),
        ]);
    }
}
