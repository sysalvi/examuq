<?php

namespace App\Http\Controllers\Web;

use App\Http\Controllers\Controller;
use App\Models\AuditLog;
use App\Models\Exam;
use App\Models\ExamSession;
use App\Models\LaunchToken;
use Carbon\Carbon;
use Illuminate\Http\Request;
use Illuminate\Http\RedirectResponse;
use Illuminate\View\View;

class ExamGatewayController extends Controller
{
    public function confirm(Request $request): View|RedirectResponse
    {
        $launchToken = $request->attributes->get('launchToken');
        $exam = $request->attributes->get('exam');
        $examSession = $request->attributes->get('examSession');

        if (! $launchToken instanceof LaunchToken || ! $exam instanceof Exam || ! $examSession instanceof ExamSession) {
            return redirect()->route('exam.block');
        }

        AuditLog::query()->create([
            'action' => 'exam_gateway_open_confirm',
            'entity_type' => 'launch_token',
            'entity_id' => (string) $launchToken->id,
            'metadata_json' => [
                'exam_id' => $exam->id,
                'session_id' => $launchToken->exam_session_id,
            ],
        ]);

        return view('exam.confirm', [
            'exam' => $exam,
            'examSession' => $examSession,
            'launchToken' => $launchToken,
        ]);
    }

    public function redirectToExam(Request $request): View|RedirectResponse
    {
        $launchToken = $request->attributes->get('launchToken');
        $exam = $request->attributes->get('exam');
        $examSession = $request->attributes->get('examSession');

        if (! $launchToken instanceof LaunchToken || ! $exam instanceof Exam || ! $examSession instanceof ExamSession || ! $exam->exam_url) {
            return redirect()->route('exam.block');
        }

        if (! $launchToken->used_at) {
            $launchToken->update([
                'used_at' => Carbon::now(),
            ]);
        }

        $deadlineAt = null;

        if ($exam->end_at) {
            $deadlineAt = $exam->end_at->copy();
        } elseif ($exam->duration_minutes && $examSession->started_at) {
            $deadlineAt = $examSession->started_at->copy()->addMinutes((int) $exam->duration_minutes);
        }

        if ($deadlineAt && $deadlineAt->lte(Carbon::now())) {
            $deadlineAt = Carbon::now()->addMinutes(120);
        }

        AuditLog::query()->create([
            'action' => 'exam_gateway_open_player',
            'entity_type' => 'launch_token',
            'entity_id' => (string) $launchToken->id,
            'metadata_json' => [
                'exam_id' => $exam->id,
                'session_id' => $launchToken->exam_session_id,
                'target_url' => $exam->exam_url,
                'deadline_at' => $deadlineAt?->toIso8601String(),
            ],
        ]);

        return view('exam.player', [
            'exam' => $exam,
            'examSession' => $examSession,
            'launchToken' => $launchToken,
            'deadlineAtIso' => $deadlineAt?->toIso8601String(),
            'serverNowIso' => Carbon::now()->toIso8601String(),
        ]);
    }

    public function blocked(): View
    {
        return view('exam.blocked');
    }
}
